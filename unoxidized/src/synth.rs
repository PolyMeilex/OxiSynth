mod public;

pub mod channel;
pub mod generator;
pub mod modulator;
pub mod soundfont;
pub mod voice_pool;

use crate::chorus::Chorus;
use crate::reverb::Reverb;
use channel::{Channel, InterpMethod};

use self::soundfont::{Preset, SoundFont};

use voice_pool::VoicePool;

use super::settings::{Settings, SettingsError, SynthDescriptor};
use super::tuning::Tuning;
use std::convert::TryInto;

#[derive(Copy, Clone)]
pub struct BankOffset {
    pub sfont_id: usize,
    pub offset: u32,
}

#[derive(Clone)]
pub(crate) struct FxBuf {
    pub reverb: [f32; 64],
    pub chorus: [f32; 64],
}

pub struct Synth {
    pub(crate) ticks: u32,

    sfont: Vec<SoundFont>,
    sfont_id: usize,

    bank_offsets: Vec<BankOffset>,

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

    tuning: Vec<Vec<Option<Tuning>>>,
    pub(crate) min_note_length_ticks: u32,

    pub(crate) settings: Settings,

    #[cfg(feature = "i16-out")]
    dither_index: i32,
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
            sfont: Vec::new(),
            sfont_id: 0 as _,
            bank_offsets: Vec::new(),
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
            tuning: Vec::new(),
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

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.settings.sample_rate = sample_rate;
        self.voices.set_sample_rate(sample_rate);

        self.chorus = Chorus::new(sample_rate, self.chorus.active);
    }

    pub(crate) fn get_preset(
        &mut self,
        sfontnum: usize,
        banknum: u32,
        prognum: u8,
    ) -> Option<Preset> {
        let sfont = self.get_sfont_by_id(sfontnum);
        if let Some(sfont) = sfont {
            let offset = self
                .get_bank_offset(sfontnum)
                .map(|o| o.offset)
                .unwrap_or_default();
            let preset = sfont.get_preset(banknum.wrapping_sub(offset as u32), prognum);
            preset
        } else {
            None
        }
    }

    pub(crate) fn find_preset(&self, banknum: u32, prognum: u8) -> Option<Preset> {
        for sfont in self.sfont.iter() {
            let offset = self
                .get_bank_offset(sfont.id)
                .map(|o| o.offset)
                .unwrap_or_default();

            let preset = sfont.get_preset(banknum.wrapping_sub(offset), prognum);
            if let Some(preset) = preset {
                return Some(preset);
            }
        }
        return None;
    }

    pub(crate) fn update_presets(&mut self) {
        for id in 0..self.channels.len() {
            let sfontnum = self.channels[id].get_sfontnum();
            let banknum = self.channels[id].get_banknum();
            let prognum = self.channels[id].get_prognum();

            let preset = self.get_preset(sfontnum, banknum, prognum);
            self.channels[id].set_preset(preset);
        }
    }
}
