use super::super::channel::ChannelId;
use crate::synth::voice_pool::{Voice, VoiceAddMode, VoiceDescriptor};
use crate::synth::{modulator::Mod, Synth};

use super::{
    loader::{InstrumentZone, PresetZone},
    Sample,
};

impl Synth {
    /// noteon
    pub(crate) fn sf_noteon(&mut self, chan: u8, key: u8, vel: u8) {
        fn preset_zone_inside_range(zone: &PresetZone, key: u8, vel: u8) -> bool {
            zone.keylo <= key
                && zone.keyhi >= key
                && zone.vello <= vel as i32
                && zone.velhi >= vel as i32
        }

        fn inst_zone_inside_range(zone: &InstrumentZone, key: u8, vel: u8) -> bool {
            zone.keylo <= key
                && zone.keyhi >= key
                && zone.vello <= vel as i32
                && zone.velhi >= vel as i32
        }

        fn sample_in_rom(sample: &Sample) -> i32 {
            // sampletype & FLUID_SAMPLETYPE_ROM
            sample.sampletype & 0x8000
        }

        let preset = &self.channels[chan as usize].preset.as_ref().unwrap().data;

        // list for 'sorting' preset modulators
        let mod_list_new: Vec<Option<&Mod>> = (0..64).into_iter().map(|_| None).collect();
        use std::convert::TryInto;
        let mut mod_list: [Option<&Mod>; 64] = mod_list_new.try_into().unwrap();

        let mut global_preset_zone = &preset.global_zone;

        // run thru all the zones of this preset
        for preset_zone in preset.zones.iter() {
            // check if the note falls into the key and velocity range of this preset
            if preset_zone_inside_range(preset_zone, key, vel) {
                let inst = preset_zone.inst.as_ref().unwrap();

                let mut global_inst_zone = &inst.global_zone;

                // run thru all the zones of this instrument
                for inst_zone in inst.zones.iter() {
                    // make sure this instrument zone has a valid sample
                    let sample = &inst_zone.sample;
                    if !(sample.is_none() || sample_in_rom(&sample.as_ref().unwrap()) != 0) {
                        // check if the note falls into the key and velocity range of this instrument
                        if inst_zone_inside_range(inst_zone, key, vel) && !sample.is_none() {
                            // this is a good zone. allocate a new synthesis process and initialize it

                            // Initialize Voice
                            let init = |voice: &mut Voice| {
                                voice.add_default_mods();

                                // Instrument level, generators
                                for i in 0..GEN_LAST {
                                    use num_traits::FromPrimitive;
                                    /* SF 2.01 section 9.4 'bullet' 4:
                                     *
                                     * A generator in a local instrument zone supersedes a
                                     * global instrument zone generator.  Both cases supersede
                                     * the default generator -> voice_gen_set */
                                    if inst_zone.gen[i as usize].flags != 0 {
                                        voice.gen_set(
                                            FromPrimitive::from_u8(i as u8).unwrap(),
                                            inst_zone.gen[i as usize].val,
                                        );
                                    } else if let Some(global_inst_zone) = &global_inst_zone {
                                        if global_inst_zone.gen[i as usize].flags as i32 != 0 {
                                            voice.gen_set(
                                                FromPrimitive::from_u8(i as u8).unwrap(),
                                                global_inst_zone.gen[i as usize].val,
                                            );
                                        }
                                    } else {
                                        /* The generator has not been defined in this instrument.
                                         * Do nothing, leave it at the default.
                                         */
                                    }
                                }

                                /* global instrument zone, modulators: Put them all into a
                                 * list. */
                                let mut mod_list_count = 0;
                                if let Some(global_inst_zone) = &mut global_inst_zone {
                                    for m in global_inst_zone.mods.iter() {
                                        mod_list[mod_list_count] = Some(m);
                                        mod_list_count += 1;
                                    }
                                }

                                /* local instrument zone, modulators.
                                 * Replace modulators with the same definition in the list:
                                 * SF 2.01 page 69, 'bullet' 8
                                 */
                                for m in inst_zone.mods.iter() {
                                    /* 'Identical' modulators will be deleted by setting their
                                     *  list entry to NULL.  The list length is known, NULL
                                     *  entries will be ignored later.  SF2.01 section 9.5.1
                                     *  page 69, 'bullet' 3 defines 'identical'.  */
                                    for i in 0..mod_list_count {
                                        if !mod_list[i].is_none()
                                            && m.test_identity(
                                                mod_list[i as usize].as_ref().unwrap(),
                                            )
                                        {
                                            mod_list[i] = None;
                                        }
                                    }

                                    /* Finally add the new modulator to to the list. */
                                    mod_list[mod_list_count] = Some(m);

                                    mod_list_count += 1;
                                }

                                // Add instrument modulators (global / local) to the voice.
                                for i in 0..mod_list_count {
                                    let mod_0 = mod_list[i as usize];
                                    if !mod_0.is_none() {
                                        // disabled modulators CANNOT be skipped.

                                        /* Instrument modulators -supersede- existing (default)
                                         * modulators.  SF 2.01 page 69, 'bullet' 6 */
                                        voice.add_mod(
                                            mod_0.as_ref().unwrap(),
                                            VoiceAddMode::Overwrite,
                                        );
                                    }
                                }

                                const GEN_STARTADDROFS: u32 = 0;
                                const GEN_ENDADDROFS: u32 = 1;
                                const GEN_STARTLOOPADDROFS: u32 = 2;
                                const GEN_ENDLOOPADDROFS: u32 = 3;
                                const GEN_STARTADDRCOARSEOFS: u32 = 4;

                                const GEN_ENDADDRCOARSEOFS: u32 = 12;

                                const GEN_STARTLOOPADDRCOARSEOFS: u32 = 45;
                                const GEN_KEYNUM: u32 = 46;
                                const GEN_VELOCITY: u32 = 47;

                                const GEN_ENDLOOPADDRCOARSEOFS: u32 = 50;
                                const GEN_SAMPLEMODE: u32 = 54;
                                const GEN_EXCLUSIVECLASS: u32 = 57;
                                const GEN_OVERRIDEROOTKEY: u32 = 58;
                                const GEN_LAST: u32 = 60;

                                /* Preset level, generators */
                                for i in 0..GEN_LAST {
                                    /* SF 2.01 section 8.5 page 58: If some generators are
                                     * encountered at preset level, they should be ignored */
                                    if i != GEN_STARTADDROFS
                                        && i != GEN_ENDADDROFS
                                        && i != GEN_STARTLOOPADDROFS
                                        && i != GEN_ENDLOOPADDROFS
                                        && i != GEN_STARTADDRCOARSEOFS
                                        && i != GEN_ENDADDRCOARSEOFS
                                        && i != GEN_STARTLOOPADDRCOARSEOFS
                                        && i != GEN_KEYNUM
                                        && i != GEN_VELOCITY
                                        && i != GEN_ENDLOOPADDRCOARSEOFS
                                        && i != GEN_SAMPLEMODE
                                        && i != GEN_EXCLUSIVECLASS
                                        && i != GEN_OVERRIDEROOTKEY
                                    {
                                        /* SF 2.01 section 9.4 'bullet' 9: A generator in a
                                         * local preset zone supersedes a global preset zone
                                         * generator.  The effect is -added- to the destination
                                         * summing node -> voice_gen_incr */
                                        if preset_zone.gen[i as usize].flags != 0 {
                                            voice.gen_incr(i, preset_zone.gen[i as usize].val);
                                        } else if let Some(global_preset_zone) = &global_preset_zone
                                        {
                                            if global_preset_zone.gen[i as usize].flags != 0 {
                                                voice.gen_incr(
                                                    i,
                                                    global_preset_zone.gen[i as usize].val,
                                                );
                                            }
                                        } else {
                                            /* The generator has not been defined in this preset
                                             * Do nothing, leave it unchanged.
                                             */
                                        }
                                    } /* if available at preset level */
                                } /* for all generators */

                                /* Global preset zone, modulators: put them all into a
                                 * list. */
                                let mut mod_list_count = 0;
                                if let Some(global_preset_zone) = &mut global_preset_zone {
                                    for m in global_preset_zone.mods.iter() {
                                        mod_list[mod_list_count] = Some(m);
                                        mod_list_count += 1;
                                    }
                                }

                                /* Process the modulators of the local preset zone.  Kick
                                 * out all identical modulators from the global preset zone
                                 * (SF 2.01 page 69, second-last bullet) */
                                for m in preset_zone.mods.iter() {
                                    for i in 0..mod_list_count {
                                        if !mod_list[i].is_none()
                                            && m.test_identity(
                                                mod_list[i as usize].as_ref().unwrap(),
                                            )
                                        {
                                            mod_list[i] = None;
                                        }
                                    }

                                    /* Finally add the new modulator to the list. */
                                    mod_list[mod_list_count] = Some(m);

                                    mod_list_count += 1;
                                }

                                // Add preset modulators (global / local) to the voice.
                                for i in 0..mod_list_count {
                                    if let Some(m) = mod_list[i] {
                                        if m.amount != 0.0 {
                                            // disabled modulators can be skipped.

                                            /* Preset modulators -add- to existing instrument /
                                             * default modulators.  SF2.01 page 70 first bullet on
                                             * page */
                                            voice.add_mod(m, VoiceAddMode::Add);
                                        }
                                    }
                                }

                                /* Store the ID of the first voice that was created by this noteon event.
                                 * Exclusive class may only terminate older voices.
                                 * That avoids killing voices, which have just been created.
                                 * (a noteon event can create several voice processes with the same exclusive
                                 * class - for example when using stereo samples)
                                 */
                            };

                            let desc = VoiceDescriptor {
                                sample: sample.as_ref().unwrap().clone(),
                                channel: &self.channels[chan as usize],
                                channel_id: ChannelId(chan as usize),
                                key,
                                vel,
                                id: self.storeid,
                                start_time: self.ticks,
                                gain: self.settings.gain,
                            };

                            let voice_id = self.voices.request_new_voice(self.noteid, desc, init);

                            if let Ok(voice_id) = voice_id {
                                log::trace!(
                                    "noteon\t{}\t{}\t{}\t{}\t{}",
                                    chan,
                                    key,
                                    vel,
                                    self.storeid,
                                    self.ticks as f32 / 44100.0,
                                );

                                // add the synthesis process to the synthesis loop.
                                self.voices.start_voice(&self.channels, voice_id);
                            } else {
                                log::warn!(
                                    "Failed to allocate a synthesis process. (chan={},key={})",
                                    chan,
                                    key
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
