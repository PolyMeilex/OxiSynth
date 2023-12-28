pub(crate) mod internal;
mod public;

pub(crate) mod channel_pool;
pub(crate) use channel_pool::Channel;
pub(crate) mod voice_pool;

mod conv;
pub use channel_pool::InterpolationMethod;

pub mod font_bank;

use super::chorus::Chorus;
use super::midi_event::MidiEvent;
use super::reverb::Reverb;
use super::OxiError;

pub mod soundfont;

use voice_pool::VoicePool;

use self::channel_pool::ChannelPool;
use self::font_bank::FontBank;

use super::settings::{Settings, SettingsError, SynthDescriptor};
use std::convert::TryInto;

#[derive(Clone)]
struct FxBuf {
    pub reverb: [f32; 64],
    pub chorus: [f32; 64],
}

pub struct Synth {
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
            (settings.min_note_length as f32 * settings.sample_rate / 1000.0) as usize;

        let nbuf = if settings.audio_groups > settings.audio_channels {
            settings.audio_groups
        } else {
            settings.audio_channels
        };

        let midi_channels = settings.midi_channels;
        let mut synth = Self {
            ticks: 0,

            font_bank: FontBank::new(),

            channels: ChannelPool::new(midi_channels as usize, None),
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

            reverb: Reverb::new(reverb_active),
            chorus: Chorus::new(settings.sample_rate, chorus_active),

            cur: 64,
            min_note_length_ticks,

            settings,

            #[cfg(feature = "i16-out")]
            dither_index: 0,
        };

        if synth.settings.drums_channel_active {
            internal::midi::bank_select(&mut synth.channels[9], 128);
        }

        Ok(synth)
    }

    pub fn send_event(&mut self, event: MidiEvent) -> Result<(), OxiError> {
        match event.check()? {
            MidiEvent::NoteOn { channel, key, vel } => {
                internal::midi::noteon(
                    self.channels.get(channel as usize)?,
                    &mut self.voices,
                    self.ticks,
                    self.min_note_length_ticks,
                    self.settings.gain,
                    key,
                    vel,
                )?;
            }
            MidiEvent::NoteOff { channel, key } => {
                internal::midi::noteoff(
                    self.channels.get(channel as usize)?,
                    &mut self.voices,
                    self.min_note_length_ticks,
                    key,
                );
            }
            MidiEvent::ControlChange {
                channel,
                ctrl,
                value,
            } => {
                internal::midi::cc(
                    self.channels.get_mut(channel as usize)?,
                    &mut self.voices,
                    self.min_note_length_ticks,
                    self.settings.drums_channel_active,
                    ctrl,
                    value,
                );
            }
            MidiEvent::AllNotesOff { channel } => {
                internal::midi::all_notes_off(
                    self.channels.get_mut(channel as usize)?,
                    &mut self.voices,
                    self.min_note_length_ticks,
                );
            }
            MidiEvent::AllSoundOff { channel } => {
                internal::all_sounds_off(channel as usize, &mut self.voices);
            }
            MidiEvent::PitchBend { channel, value } => {
                internal::pitch_bend(
                    self.channels.get_mut(channel as usize)?,
                    &mut self.voices,
                    value,
                );
            }
            MidiEvent::ProgramChange {
                channel,
                program_id,
            } => {
                internal::midi::program_change(
                    self.channels.get_mut(channel as usize)?,
                    &self.font_bank,
                    program_id,
                    self.settings.drums_channel_active,
                );
            }
            MidiEvent::ChannelPressure { channel, value } => {
                internal::midi::channel_pressure(
                    self.channels.get_mut(channel as usize)?,
                    &mut self.voices,
                    value,
                );
            }
            MidiEvent::PolyphonicKeyPressure {
                channel,
                key,
                value,
            } => {
                internal::midi::key_pressure(
                    self.channels.get_mut(channel as usize)?,
                    &mut self.voices,
                    key,
                    value,
                );
            }
            MidiEvent::SystemReset => {
                internal::midi::system_reset(
                    &mut self.voices,
                    &mut self.channels,
                    &self.font_bank,
                    &mut self.chorus,
                    &mut self.reverb,
                );
            }
        };

        Ok(())
    }
}
