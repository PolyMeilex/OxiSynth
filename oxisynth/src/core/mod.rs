pub(crate) mod midi;
pub(crate) mod write;

mod channel_pool;
use channel_pool::Channel;
mod settings;
mod voice_pool;

mod conv;
pub use channel_pool::InterpolationMethod;
pub(crate) use settings::Settings;

mod font_bank;

use oxisynth_chorus::Chorus;
use oxisynth_reverb::Reverb;

mod soundfont;
pub use soundfont::{generator::GeneratorType, Preset, SoundFont};

use voice_pool::VoicePool;
use write::OutputBuffer;

use self::channel_pool::ChannelPool;
use self::font_bank::FontBank;

use crate::{SettingsError, SynthDescriptor};

pub(crate) struct Core {
    ticks: usize,
    pub font_bank: FontBank,

    pub channels: ChannelPool,
    pub voices: VoicePool,

    pub reverb: Reverb,
    pub chorus: Chorus,

    pub settings: Settings,

    output: OutputBuffer,
}

impl Default for Core {
    fn default() -> Self {
        Self::new(Default::default()).unwrap()
    }
}

impl Core {
    pub fn new(desc: SynthDescriptor) -> Result<Self, SettingsError> {
        let settings: Settings = desc.try_into()?;

        let nbuf = if settings.audio_groups > settings.audio_channels {
            settings.audio_groups
        } else {
            settings.audio_channels
        };

        let mut synth = Self {
            ticks: 0,

            font_bank: FontBank::new(),

            channels: ChannelPool::new(settings.midi_channels as usize),
            voices: VoicePool::new(settings.polyphony as usize, settings.sample_rate),

            output: OutputBuffer::new(nbuf as usize),

            reverb: Reverb::new(),
            chorus: Chorus::new(settings.sample_rate),

            settings,
        };

        if synth.settings.drums_channel_active {
            synth.channels[9].set_banknum(128);
        }

        Ok(synth)
    }
}
