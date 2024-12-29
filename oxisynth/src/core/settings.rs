use crate::{RangeError, SettingsError, SynthDescriptor};

pub(crate) struct Settings {
    pub reverb_active: bool,
    pub chorus_active: bool,
    pub drums_channel_active: bool,

    /// Def: 256
    /// Min: 1
    /// Max: 65535
    pub polyphony: u16,
    /// Def: 16
    /// Min: 16
    /// Max: 256
    pub midi_channels: u8,
    /// Def: 0.2
    /// Min: 0.0
    /// Max: 10.0
    pub gain: f32,
    /// Def: 1
    /// Min: 1
    /// Max: 128
    pub audio_channels: u8,
    /// Def: 1
    /// Min: 1
    /// Max: 128
    pub audio_groups: u8,
    /// Def: 44100.0
    /// Min: 8000.0
    /// Max: 96000.0
    pub sample_rate: f32,
    /// Def: 10
    /// Min: 0
    /// Max: 65535
    pub min_note_length: u16,
}

struct Range<T> {
    min: T,
    max: T,
}

impl<T: PartialOrd + Copy> Range<T> {
    fn check(&self, v: T) -> Result<T, RangeError<T>> {
        if v < self.min {
            Err(RangeError::ToSmall {
                got: v,
                min: self.min,
            })
        } else if v > self.max {
            Err(RangeError::ToBig {
                got: v,
                max: self.max,
            })
        } else {
            Ok(v)
        }
    }
}

static POLYPHONY_RANGE: Range<u16> = Range { min: 1, max: 65535 };
static GAIN_RANGE: Range<f32> = Range {
    min: 0.0,
    max: 10.0,
};
static AUDIO_CHANNELS_RANGE: Range<u8> = Range { min: 1, max: 128 };
static AUDIO_GROUPS_RANGE: Range<u8> = Range { min: 1, max: 128 };
static SAMPLE_RATE_RANGE: Range<f32> = Range {
    min: 8000.0,
    max: 96000.0,
};

impl TryFrom<SynthDescriptor> for Settings {
    type Error = SettingsError;

    fn try_from(desc: SynthDescriptor) -> Result<Self, Self::Error> {
        let midi_channels = if desc.midi_channels % 16 != 0 {
            log::warn!("Requested number of MIDI channels is not a multiple of 16. Increase the number of channels to the next multiple.");
            return Err(SettingsError::MidiChannelsIsNotMultipleOf16);
        } else {
            desc.midi_channels
        };

        let polyphony = POLYPHONY_RANGE
            .check(desc.polyphony)
            .map_err(SettingsError::PolyphonyRange)?;

        let gain = GAIN_RANGE
            .check(desc.gain)
            .map_err(SettingsError::GainRange)?;

        let audio_channels = AUDIO_CHANNELS_RANGE
            .check(desc.audio_channels)
            .map_err(SettingsError::AudioChannelRange)?;

        let audio_groups = AUDIO_GROUPS_RANGE
            .check(desc.audio_groups)
            .map_err(SettingsError::AudioGroupsRange)?;

        let sample_rate = SAMPLE_RATE_RANGE
            .check(desc.sample_rate)
            .map_err(SettingsError::SammpleRateRange)?;

        // Guarded by type system
        let min_note_length = desc.min_note_length;

        Ok(Self {
            reverb_active: desc.reverb_active,
            chorus_active: desc.chorus_active,
            drums_channel_active: desc.drums_channel_active,

            polyphony,
            midi_channels,
            gain,
            audio_channels,
            audio_groups,
            sample_rate,
            min_note_length,
        })
    }
}
