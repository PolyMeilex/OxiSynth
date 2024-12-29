pub(crate) mod midi;
pub(crate) mod write;

pub(crate) mod channel_pool;
pub(crate) use channel_pool::Channel;
mod settings;
pub(crate) mod voice_pool;

mod conv;
pub use channel_pool::InterpolationMethod;
pub(crate) use settings::Settings;

pub mod font_bank;

use oxisynth_chorus::Chorus;
use oxisynth_reverb::Reverb;

pub mod soundfont;
use soundfont::SoundFont;

use voice_pool::VoicePool;

use self::channel_pool::ChannelPool;
use self::font_bank::FontBank;

use crate::{SettingsError, SynthDescriptor};

#[derive(Clone)]
struct FxBuf {
    pub reverb: [f32; 64],
    pub chorus: [f32; 64],
}

pub(crate) struct Core {
    ticks: usize,
    pub font_bank: FontBank,

    pub channels: ChannelPool,
    pub voices: VoicePool,

    left_buf: Vec<[f32; 64]>,
    right_buf: Vec<[f32; 64]>,

    fx_left_buf: FxBuf,
    fx_right_buf: FxBuf,

    pub reverb: Reverb,
    pub chorus: Chorus,

    cur: usize,

    min_note_length_ticks: usize,

    pub settings: Settings,

    #[cfg(feature = "i16-out")]
    i16_output: write::i16_write::I16OutputState,
}

impl Default for Core {
    fn default() -> Self {
        Self::new(Default::default()).unwrap()
    }
}

impl Core {
    pub fn new(desc: SynthDescriptor) -> Result<Self, SettingsError> {
        let settings: Settings = desc.try_into()?;

        let min_note_length_ticks =
            (settings.min_note_length as f32 * settings.sample_rate / 1000.0) as usize;

        let nbuf = if settings.audio_groups > settings.audio_channels {
            settings.audio_groups
        } else {
            settings.audio_channels
        };

        let mut synth = Self {
            ticks: 0,

            font_bank: FontBank::new(),

            channels: ChannelPool::new(settings.midi_channels as usize, None),
            voices: VoicePool::new(settings.polyphony as usize, settings.sample_rate),
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

            reverb: Reverb::new(),
            chorus: Chorus::new(settings.sample_rate),

            cur: 64,
            min_note_length_ticks,

            settings,

            #[cfg(feature = "i16-out")]
            i16_output: write::i16_write::I16OutputState::default(),
        };

        if synth.settings.drums_channel_active {
            synth.channels[9].set_banknum(128);
        }

        Ok(synth)
    }
}
