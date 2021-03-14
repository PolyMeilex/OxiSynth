pub mod count;
pub mod font;
pub mod gen;
pub mod midi;
pub mod params;
pub mod reverb;
pub mod tuning;
pub mod write;

use crate::voice_pool::VoicePool;

use super::channel::{Channel, InterpMethod};
use super::chorus::Chorus;
use super::reverb::ReverbModel;
use super::settings::{Settings, SettingsError, SynthDescriptor};
use super::soundfont::Preset;
use super::soundfont::SoundFont;
use super::tuning::Tuning;
use std::convert::TryInto;

const GEN_LAST: u8 = 60;

#[derive(Copy, Clone)]
pub struct BankOffset {
    pub sfont_id: u32,
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
    sfont_id: u32,
    bank_offsets: Vec<BankOffset>,
    pub(crate) gain: f32,
    pub(crate) channel: Vec<Channel>,
    pub(crate) voices: VoicePool,
    pub(crate) noteid: usize,
    pub(crate) storeid: usize,
    nbuf: u8,

    left_buf: Vec<[f32; 64]>,
    right_buf: Vec<[f32; 64]>,

    fx_left_buf: FxBuf,
    fx_right_buf: FxBuf,

    pub reverb: ReverbModel,
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
            gain: settings.gain,
            channel: Vec::new(),
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

            reverb: ReverbModel::new(reverb_active),
            chorus: Chorus::new(settings.sample_rate, chorus_active),

            cur: 64,
            tuning: Vec::new(),
            min_note_length_ticks,

            settings,

            #[cfg(feature = "i16-out")]
            dither_index: 0,
        };

        for i in 0..synth.settings.midi_channels {
            synth.channel.push(Channel::new(&synth, i));
        }

        synth.set_reverb_params(0.2, 0.0, 0.5, 0.9);

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
        sfontnum: u32,
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
        for chan in 0..(self.settings.midi_channels as usize) {
            let sfontnum = self.channel[chan].get_sfontnum();
            let banknum = self.channel[chan].get_banknum();
            let prognum = self.channel[chan].get_prognum();
            let preset = self.get_preset(sfontnum, banknum, prognum);
            self.channel[chan].set_preset(preset);
        }
    }
}
