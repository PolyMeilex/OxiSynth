mod public;

pub mod bank;

mod channel_pool;
pub(crate) mod modulator;
pub(crate) mod voice_pool;

pub mod generator;
pub use channel_pool::InterpolationMethod;

mod conv;
mod font_bank;

use crate::chorus::Chorus;
use crate::reverb::Reverb;

use crate::soundfont::Preset;

use voice_pool::VoicePool;

use self::channel_pool::ChannelPool;
use self::font_bank::FontBank;

use super::settings::{Settings, SettingsError, SynthDescriptor};
use std::convert::TryInto;

#[derive(Clone)]
pub(crate) struct FxBuf {
    pub reverb: [f32; 64],
    pub chorus: [f32; 64],
}

pub struct Synth {
    ticks: u32,

    pub font_bank: FontBank,

    channels: ChannelPool,
    voices: VoicePool,

    nbuf: u8,

    left_buf: Vec<[f32; 64]>,
    right_buf: Vec<[f32; 64]>,

    fx_left_buf: FxBuf,
    fx_right_buf: FxBuf,

    pub reverb: Reverb,
    pub chorus: Chorus,

    cur: usize,

    min_note_length_ticks: u32,

    settings: Settings,

    #[cfg(feature = "i16-out")]
    dither_index: i32,
}

impl Default for Synth {
    fn default() -> Self {
        Self::new(Default::default()).unwrap()
    }
}

impl Synth {
    pub fn new(desc: SynthDescriptor) -> Result<Self, SettingsError> {
        let chorus_active = desc.chorus_active;
        let reverb_active = desc.reverb_active;

        let settings: Settings = desc.try_into()?;

        let min_note_length_ticks =
            (settings.min_note_length as f32 * settings.sample_rate / 1000.0) as u32;

        let nbuf = {
            let nbuf = settings.audio_channels;
            if settings.audio_groups > nbuf {
                settings.audio_groups
            } else {
                nbuf
            }
        };

        let midi_channels = settings.midi_channels;
        let mut synth = Self {
            ticks: 0,

            font_bank: FontBank::new(),

            channels: ChannelPool::new(midi_channels as usize, None),
            voices: VoicePool::new(settings.polyphony as usize, settings.sample_rate),
            nbuf,
            left_buf: vec![[0.0; 64]; nbuf as usize],
            right_buf: vec![[0.0; 64]; nbuf as usize],

            fx_left_buf: FxBuf {
                reverb: [0.0; 64],
                chorus: [0.0; 64],
            },
            fx_right_buf: FxBuf {
                reverb: [0.0; 64],
                chorus: [0.0; 64],
            },

            reverb: Reverb::new(reverb_active),
            chorus: Chorus::new(settings.sample_rate, chorus_active),

            cur: 64,
            min_note_length_ticks,

            settings,

            #[cfg(feature = "i16-out")]
            dither_index: 0,
        };

        if synth.settings.drums_channel_active {
            synth.bank_select(9, 128).ok();
        }

        Ok(synth)
    }
}

use modulator::Mod;
use voice_pool::{Voice, VoiceAddMode, VoiceDescriptor};

use crate::soundfont::{InstrumentZone, PresetZone};

impl Synth {
    fn sf_noteon(&mut self, chan: usize, key: u8, vel: u8) {
        fn preset_zone_inside_range(zone: &PresetZone, key: u8, vel: u8) -> bool {
            zone.key_low <= key
                && zone.key_high >= key
                && zone.vel_low <= vel
                && zone.vel_high >= vel
        }

        fn inst_zone_inside_range(zone: &InstrumentZone, key: u8, vel: u8) -> bool {
            zone.key_low <= key
                && zone.key_high >= key
                && zone.vel_low <= vel
                && zone.vel_high >= vel
        }

        let preset = &self.channels[chan].preset().unwrap();

        // list for 'sorting' preset modulators
        let mod_list_new: Vec<Option<&Mod>> = (0..64).into_iter().map(|_| None).collect();
        let mut mod_list: [Option<&Mod>; 64] = mod_list_new.try_into().unwrap();

        let mut global_preset_zone = preset.global_zone();

        // run thru all the zones of this preset
        for preset_zone in preset.zones().iter() {
            // check if the note falls into the key and velocity range of this preset
            if preset_zone_inside_range(preset_zone, key, vel) {
                let inst = preset_zone.inst.as_ref().unwrap();

                let mut global_inst_zone = &inst.global_zone();

                // run thru all the zones of this instrument
                for inst_zone in inst.zones().iter() {
                    // make sure this instrument zone has a valid sample
                    let sample = &inst_zone.sample;
                    if !(sample.is_none() || sample.as_ref().unwrap().sample_type.is_rom()) {
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
                                        if mod_list[i].is_some()
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
                                    if mod_0.is_some() {
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
                                        if mod_list[i].is_some()
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
                                channel_id: chan,
                                key,
                                vel,
                                start_time: self.ticks,
                                gain: self.settings.gain,
                            };

                            let voice_id = self.voices.request_new_voice(
                                &self.channels[chan as usize],
                                desc,
                                init,
                            );

                            if let Ok(_) = voice_id {
                                log::trace!(
                                    "noteon\t{}\t{}\t{}\t\t{}",
                                    chan,
                                    key,
                                    vel,
                                    self.ticks as f32 / 44100.0,
                                );
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
