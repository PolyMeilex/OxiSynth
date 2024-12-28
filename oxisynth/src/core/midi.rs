use std::convert::{TryFrom, TryInto};

use crate::core::channel_pool::Channel;
use crate::core::font_bank::FontBank;
use crate::core::soundfont::modulator::Mod;
use crate::core::soundfont::{
    generator::{gen_scale_nrpn, GeneratorType},
    InstrumentZone, PresetZone,
};
use crate::core::voice_pool::{Voice, VoiceAddMode, VoiceDescriptor, VoicePool};
use crate::OxiError;

type MidiControlChange = u32;
const RPN_MSB: MidiControlChange = 101;
const RPN_LSB: MidiControlChange = 100;
const NRPN_MSB: MidiControlChange = 99;
const NRPN_LSB: MidiControlChange = 98;
const DATA_ENTRY_LSB: MidiControlChange = 38;

/// Change the value of a generator. This function allows to control
/// all synthesis parameters in real-time. The changes are additive,
/// i.e. they add up to the existing parameter value. This function is
/// similar to sending an NRPN message to the synthesizer. The
/// function accepts a float as the value of the parameter. The
/// parameter numbers and ranges are described in the SoundFont 2.01
/// specification, paragraph 8.1.3, page 48.
pub(crate) fn set_gen(
    channel: &mut Channel,
    voices: &mut VoicePool,
    param: GeneratorType,
    value: f32,
) {
    channel.set_gen(param, value);
    channel.set_gen_abs(param, 0);

    voices.set_gen(channel.id(), param, value);
}

/// Send a noteon message.
pub(in super::super) fn noteon(
    channel: &Channel,
    voices: &mut VoicePool,
    start_time: usize,
    min_note_length_ticks: usize,
    gain: f32,
    key: u8,
    vel: u8,
) -> Result<(), OxiError> {
    if vel == 0 {
        voices.noteoff(channel, min_note_length_ticks, key);
        Ok(())
    } else if channel.preset().is_none() {
        Err(OxiError::ChannelHasNoPreset)
    } else {
        voices.release_voice_on_same_note(channel, key, min_note_length_ticks);
        voices.noteid_add();

        inner_noteon(channel, voices, start_time, gain, key, vel);
        Ok(())
    }
}

fn inner_noteon(
    channel: &Channel,
    voices: &mut VoicePool,
    start_time: usize,
    gain: f32,
    key: u8,
    vel: u8,
) {
    fn preset_zone_inside_range(zone: &PresetZone, key: u8, vel: u8) -> bool {
        zone.key_low <= key && zone.key_high >= key && zone.vel_low <= vel && zone.vel_high >= vel
    }

    fn inst_zone_inside_range(zone: &InstrumentZone, key: u8, vel: u8) -> bool {
        zone.key_low <= key && zone.key_high >= key && zone.vel_low <= vel && zone.vel_high >= vel
    }

    let preset = &channel.preset().unwrap();

    // list for 'sorting' preset modulators
    let mod_list_new: Vec<Option<&Mod>> = (0..64).map(|_| None).collect();
    let mut mod_list: [Option<&Mod>; 64] = mod_list_new.try_into().unwrap();

    let mut global_preset_zone = preset.global_zone();

    // run thru all the zones of this preset
    for preset_zone in preset.zones().iter() {
        // check if the note falls into the key and velocity range of this preset
        if !preset_zone_inside_range(preset_zone, key, vel) {
            continue;
        }

        let Some(inst) = preset_zone.inst.as_ref() else {
            log::error!("Instrument for zone: {:?} is missing", preset_zone.name);
            continue;
        };

        let mut global_inst_zone = &inst.global_zone();

        // run thru all the zones of this instrument
        for inst_zone in inst.zones().iter() {
            let Some(sample) = inst_zone.sample.as_ref() else {
                continue;
            };

            if sample.sample_type().is_rom() {
                continue;
            }

            // check if the note falls into the key and velocity range of this instrument
            if !inst_zone_inside_range(inst_zone, key, vel) {
                continue;
            }

            // this is a good zone. allocate a new synthesis process and initialize it

            // Initialize Voice
            let init = |voice: &mut Voice| {
                voice.add_default_mods();

                // Instrument level, generators
                for gen in GeneratorType::iter() {
                    // SF 2.01 section 9.4 'bullet' 4:
                    //
                    // A generator in a local instrument zone supersedes a
                    // global instrument zone generator.  Both cases supersede
                    // the default generator -> voice_gen_set
                    if inst_zone.gen[gen].flags != 0 {
                        voice.gen_set(gen, inst_zone.gen[gen].val);
                    } else if let Some(global_inst_zone) = &global_inst_zone {
                        if global_inst_zone.gen[gen].flags as i32 != 0 {
                            voice.gen_set(gen, global_inst_zone.gen[gen].val);
                        }
                    } else {
                        // The generator has not been defined in this instrument.
                        // Do nothing, leave it at the default.
                    }
                }

                // global instrument zone, modulators: Put them all into a
                // list.
                let mut mod_list_count = 0;
                if let Some(global_inst_zone) = &mut global_inst_zone {
                    for m in global_inst_zone.mods.iter() {
                        mod_list[mod_list_count] = Some(m);
                        mod_list_count += 1;
                    }
                }

                // local instrument zone, modulators.
                // Replace modulators with the same definition in the list:
                // SF 2.01 page 69, 'bullet' 8
                for m in inst_zone.mods.iter() {
                    // 'Identical' modulators will be deleted by setting their
                    //  list entry to None. The list length is known, None
                    //  entries will be ignored later. SF2.01 section 9.5.1
                    //  page 69, 'bullet' 3 defines 'identical'.
                    mod_list
                        .iter_mut()
                        .take(mod_list_count)
                        .filter(|modulator| {
                            modulator
                                .as_ref()
                                .map(|modulator| m.test_identity(modulator))
                                .unwrap_or(false)
                        })
                        .for_each(|modulator| {
                            *modulator = None;
                        });

                    // Finally add the new modulator to to the list.
                    mod_list[mod_list_count] = Some(m);
                    mod_list_count += 1;
                }

                // Add instrument modulators (global / local) to the voice.
                mod_list
                    .iter()
                    .take(mod_list_count)
                    .flatten()
                    .for_each(|modulator| {
                        // disabled modulators CANNOT be skipped.

                        // Instrument modulators -supersede- existing (default)
                        // modulators.  SF 2.01 page 69, 'bullet' 6
                        voice.add_mod(modulator, VoiceAddMode::Overwrite);
                    });

                // Preset level, generators
                for gen in GeneratorType::iter() {
                    // SF 2.01 section 8.5 page 58: If some generators are
                    // encountered at preset level, they should be ignored
                    if matches!(
                        gen,
                        GeneratorType::StartAddrOfs
                            | GeneratorType::EndAddrOfs
                            | GeneratorType::StartLoopAddrOfs
                            | GeneratorType::EndLoopAddrOfs
                            | GeneratorType::StartAddrCoarseOfs
                            | GeneratorType::EndAddrCoarseOfs
                            | GeneratorType::StartLoopAddrCoarseOfs
                            | GeneratorType::KeyNum
                            | GeneratorType::Velocity
                            | GeneratorType::EndLoopAddrCoarseOfs
                            | GeneratorType::SampleMode
                            | GeneratorType::ExclusiveClass
                            | GeneratorType::OverrideRootKey
                    ) {
                        continue;
                    }

                    // SF 2.01 section 9.4 'bullet' 9: A generator in a
                    // local preset zone supersedes a global preset zone
                    // generator.  The effect is -added- to the destination
                    // summing node -> voice_gen_incr
                    if preset_zone.gen[gen].flags != 0 {
                        voice.gen_incr(gen, preset_zone.gen[gen].val);
                    } else if let Some(global_preset_zone) = &global_preset_zone {
                        if global_preset_zone.gen[gen].flags != 0 {
                            voice.gen_incr(gen, global_preset_zone.gen[gen].val);
                        }
                    } else {
                        // The generator has not been defined in this preset
                        // Do nothing, leave it unchanged.
                    }
                }

                // Global preset zone, modulators: put them all into a list.
                let mut mod_list_count = 0;
                if let Some(global_preset_zone) = &mut global_preset_zone {
                    for m in global_preset_zone.mods.iter() {
                        mod_list[mod_list_count] = Some(m);
                        mod_list_count += 1;
                    }
                }

                // Process the modulators of the local preset zone. Kick
                // out all identical modulators from the global preset zone
                // (SF 2.01 page 69, second-last bullet)
                for m in preset_zone.mods.iter() {
                    mod_list
                        .iter_mut()
                        .take(mod_list_count)
                        .filter(|modulator| {
                            modulator
                                .as_ref()
                                .map(|modulator| m.test_identity(modulator))
                                .unwrap_or(false)
                        })
                        .for_each(|modulator| {
                            *modulator = None;
                        });

                    // Finally add the new modulator to the list.
                    mod_list[mod_list_count] = Some(m);
                    mod_list_count += 1;
                }

                // Add preset modulators (global / local) to the voice.
                mod_list
                    .iter()
                    .take(mod_list_count)
                    .flatten()
                    // disabled modulators can be skipped.
                    .filter(|m| m.amount != 0.0)
                    .for_each(|m| {
                        // Preset modulators -add- to existing instrument
                        // default modulators. SF2.01 page 70 first bullet on
                        // page
                        voice.add_mod(m, VoiceAddMode::Add);
                    });

                // Store the ID of the first voice that was created by this noteon event.
                // Exclusive class may only terminate older voices.
                // That avoids killing voices, which have just been created.
                // (a noteon event can create several voice processes with the same exclusive
                // class - for example when using stereo samples)
            };

            let desc = VoiceDescriptor {
                sample: sample.clone(),
                channel,
                key,
                vel,
                start_time,
                gain,
            };

            let voice_id = voices.request_new_voice(desc, init);

            if voice_id.is_ok() {
                log::trace!(
                    "noteon\t{}\t{}\t{}\t\t{}",
                    channel.id(),
                    key,
                    vel,
                    start_time as f32 / 44100.0,
                );
            } else {
                log::warn!(
                    "Failed to allocate a synthesis process. (chan={},key={})",
                    channel.id(),
                    key
                );
            }
        }
    }
}

/// Send a control change message.
pub(in super::super) fn cc(
    channel: &mut Channel,
    voices: &mut VoicePool,
    min_note_length_ticks: usize,
    drums_channel_active: bool,
    num: u8,
    value: u8,
) {
    *channel.cc_mut(num as usize) = value;

    match num {
        // SUSTAIN_SWITCH
        64 => {
            if value < 64 {
                // sustain off
                voices.damp_voices(channel, min_note_length_ticks)
            } else {
                // sustain on
            }
        }

        // BANK_SELECT_MSB
        0 => {
            if channel.id() == 9 && drums_channel_active {
                // ignored
                return;
            }

            channel.set_bank_msb(value & 0x7f);

            /* I fixed the handling of a MIDI bank select controller 0,
            e.g., bank select MSB (or "coarse" bank select according to
            my spec).  Prior to this fix a channel's bank number was only
            changed upon reception of MIDI bank select controller 32,
            e.g, bank select LSB (or "fine" bank-select according to my
            spec). [KLE]
            FIXME: is this correct? [PH] */
            channel.set_banknum((value & 0x7f) as u32);
        }

        // BANK_SELECT_LSB
        32 => {
            if channel.id() == 9 && drums_channel_active {
                // ignored
                return;
            }

            /* FIXME: according to the Downloadable Sounds II specification,
            bit 31 should be set when we receive the message on channel
            10 (drum channel) */
            channel
                .set_banknum((value as u32 & 0x7f).wrapping_add((channel.bank_msb() as u32) << 7));
        }

        // ALL_NOTES_OFF
        123 => {
            voices.all_notes_off(channel, min_note_length_ticks);
        }

        // ALL_SOUND_OFF
        120 => {
            voices.all_notes_off(channel, min_note_length_ticks);
        }

        // ALL_CTRL_OFF
        121 => {
            channel.init_ctrl(1);
            voices.modulate_voices_all(channel);
        }

        // DATA_ENTRY_MSB
        6 => {
            let data: i32 = ((value as i32) << 7) + channel.cc(DATA_ENTRY_LSB as usize) as i32;

            if channel.nrpn_active() != 0 {
                let (nrpn_select, nrpn_msb, nrpn_lsb) = (
                    channel.nrpn_select(),
                    channel.cc(NRPN_MSB as usize),
                    channel.cc(NRPN_LSB as usize),
                );

                // SontFont 2.01 NRPN Message (Sect. 9.6, p. 74)
                if nrpn_msb == 120 && nrpn_lsb < 100 {
                    if (nrpn_select as i32) < GeneratorType::last() as i32 {
                        let scale_nrpn: f32 = gen_scale_nrpn(nrpn_select, data);

                        let param = GeneratorType::try_from(nrpn_select as u8).unwrap();
                        set_gen(channel, voices, param, scale_nrpn)
                    }

                    channel.set_nrpn_select(0); // Reset to 0
                }
            }
            /* RPN is active: MSB = 0? */
            else if channel.cc(RPN_MSB as usize) == 0 {
                match channel.cc(RPN_LSB as usize) {
                    // RPN_PITCH_BEND_RANGE
                    0 => pitch_wheel_sens(channel, voices, value),
                    // RPN_CHANNEL_FINE_TUNE
                    1 => {
                        // Fine tune is 14 bit over +/-1 semitone (+/- 100 cents, 8192 = center)
                        set_gen(
                            channel,
                            voices,
                            GeneratorType::FineTune,
                            ((data - 8192) as f64 / 8192.0f64 * 100.0f64) as f32,
                        );
                    }
                    // RPN_CHANNEL_COARSE_TUNE
                    2 => {
                        // Coarse tune is 7 bit and in semitones (64 is center)
                        set_gen(
                            channel,
                            voices,
                            GeneratorType::CoarseTune,
                            (value - 64) as f32,
                        );
                    }
                    // TODO: This is fishy, for some reason those are missing from FluidLite, but
                    // are pressent in Fluidsynth
                    // https://github.com/FluidSynth/fluidsynth/blob/fa5173cbaefed60121db057bad7be7686165f7cc/src/synth/fluid_synth.c#L1857

                    // RPN_TUNING_PROGRAM_CHANGE | RPN_TUNING_BANK_SELECT | RPN_MODULATION_DEPTH_RANGE
                    // 3 | 4 | 5 => {}
                    _ => {}
                }
            }
        }

        // NRPN_MSB
        99 => {
            *channel.cc_mut(NRPN_LSB as usize) = 0;
            channel.set_nrpn_select(0);
            channel.set_nrpn_active(1);
        }

        // NRPN_LSB
        98 => {
            // SontFont 2.01 NRPN Message (Sect. 9.6, p. 74)
            if channel.cc(NRPN_MSB as usize) == 120 {
                if value == 100 {
                    channel.set_nrpn_select(channel.nrpn_select() + 100);
                } else if value == 101 {
                    channel.set_nrpn_select(channel.nrpn_select() + 1000);
                } else if value == 102 {
                    channel.set_nrpn_select(channel.nrpn_select() + 10000);
                } else if value < 100 {
                    channel.set_nrpn_select(channel.nrpn_select() + value as i16);
                }
            }
            channel.set_nrpn_active(1);
        }

        // RPN_MSB | RPN_LSB
        101 | 100 => channel.set_nrpn_active(0),
        _ => voices.modulate_voices(channel, true, num),
    }
}

/// Set the pitch wheel sensitivity.
pub(crate) fn pitch_wheel_sens(channel: &mut Channel, voices: &mut VoicePool, val: u8) {
    const MOD_PITCHWHEELSENS: u8 = 16;
    channel.set_pitch_wheel_sensitivity(val);
    voices.modulate_voices(channel, false, MOD_PITCHWHEELSENS);
}

/// Send a program change message.
pub(crate) fn program_change(
    channel: &mut Channel,
    font_bank: &FontBank,
    program_id: u8,
    drums_channel_active: bool,
) {
    let banknum = channel.banknum();
    channel.set_prognum(program_id);

    let mut preset = if channel.id() == 9 && drums_channel_active {
        font_bank.find_preset(128, program_id)
    } else {
        font_bank.find_preset(banknum, program_id)
    };

    if preset.is_none() {
        let mut subst_bank = banknum as i32;
        let mut subst_prog = program_id;
        if banknum != 128 {
            subst_bank = 0;
            preset = font_bank.find_preset(0, program_id);
            if preset.is_none() && program_id != 0 {
                preset = font_bank.find_preset(0, 0);
                subst_prog = 0;
            }
        } else {
            preset = font_bank.find_preset(128, 0);
            subst_prog = 0;
        }
        if preset.is_none() {
            log::warn!(
                        "Instrument not found on channel {} [bank={} prog={}], substituted [bank={} prog={}]",
                        channel.id(), banknum, program_id,
                        subst_bank, subst_prog);
        }
    }

    channel.set_sfontnum(preset.as_ref().map(|p| p.0));
    channel.set_preset(preset.map(|p| p.1));
}
