mod public;

pub mod bank;

use bank::BankOffsets;

pub(crate) mod channel;
pub(crate) mod modulator;
pub(crate) mod voice_pool;

pub mod generator;
pub use channel::InterpolationMethod;

use crate::chorus::Chorus;
use crate::reverb::Reverb;
use channel::Channel;

use crate::soundfont::{Preset, SoundFont, SoundFontId};

use voice_pool::VoicePool;

use super::settings::{Settings, SettingsError, SynthDescriptor};
use std::convert::TryInto;

use generational_arena::Arena;

#[derive(Clone)]
pub(crate) struct FxBuf {
    pub reverb: [f32; 64],
    pub chorus: [f32; 64],
}

pub struct Synth {
    pub(crate) ticks: u32,

    fonts: Arena<SoundFont>,
    fonts_stack: Vec<SoundFontId>,

    pub bank_offsets: BankOffsets,

    pub(crate) channels: Vec<Channel>,
    pub(crate) voices: VoicePool,

    pub(crate) noteid: usize,
    pub(crate) storeid: usize,

    nbuf: u8,

    left_buf: Vec<[f32; 64]>,
    right_buf: Vec<[f32; 64]>,

    fx_left_buf: FxBuf,
    fx_right_buf: FxBuf,

    pub reverb: Reverb,
    pub chorus: Chorus,

    cur: usize,

    pub(crate) min_note_length_ticks: u32,

    pub(crate) settings: Settings,

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

        let mut synth = Self {
            ticks: 0,

            fonts: Arena::new(),
            fonts_stack: Vec::new(),

            bank_offsets: Default::default(),
            channels: Vec::new(),
            voices: VoicePool::new(settings.polyphony as usize, settings.sample_rate),
            noteid: 0,
            storeid: 0 as _,

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

        for i in 0..synth.settings.midi_channels {
            synth.channels.push(Channel::new(&synth, i));
        }

        if synth.settings.drums_channel_active {
            synth.bank_select(9, 128).ok();
        }

        Ok(synth)
    }

    pub(crate) fn get_preset(
        &mut self,
        sfont_id: SoundFontId,
        banknum: u32,
        prognum: u8,
    ) -> Option<Preset> {
        let sfont = self.get_sfont(sfont_id);
        if let Some(sfont) = sfont {
            let offset = self
                .bank_offsets
                .get(sfont_id)
                .map(|o| o.offset)
                .unwrap_or_default();
            let preset = sfont.get_preset(banknum.wrapping_sub(offset as u32), prognum);
            preset
        } else {
            None
        }
    }

    pub(crate) fn find_preset(&self, banknum: u32, prognum: u8) -> Option<(SoundFontId, Preset)> {
        for id in self.fonts_stack.iter() {
            let sfont = self.fonts.get(id.0);
            if let Some(sfont) = sfont {
                let offset = self
                    .bank_offsets
                    .get(*id)
                    .map(|o| o.offset)
                    .unwrap_or_default();

                let preset = sfont.get_preset(banknum.wrapping_sub(offset), prognum);
                if let Some(preset) = preset {
                    return Some((*id, preset));
                }
            }
        }
        None
    }

    pub(crate) fn update_presets(&mut self) {
        for id in 0..self.channels.len() {
            let sfontnum = self.channels[id].get_sfontnum();
            if let Some(sfontnum) = sfontnum {
                let banknum = self.channels[id].get_banknum();
                let prognum = self.channels[id].get_prognum();

                let preset = self.get_preset(sfontnum, banknum, prognum);
                self.channels[id].set_preset(preset);
            }
        }
    }
}

use channel::ChannelId;
use modulator::Mod;
use voice_pool::{Voice, VoiceAddMode, VoiceDescriptor};

use crate::soundfont::{
    loader::{InstrumentZone, PresetZone},
    Sample,
};

impl Synth {
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

        fn sample_in_rom(sample: &Sample) -> bool {
            sample.sampletype.is_rom()
        }

        let preset = &self.channels[chan as usize].preset.as_ref().unwrap().data;

        // list for 'sorting' preset modulators
        let mod_list_new: Vec<Option<&Mod>> = (0..64).into_iter().map(|_| None).collect();
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
                    if !(sample.is_none() || sample_in_rom(&sample.as_ref().unwrap())) {
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
