#![forbid(unsafe_code)]

use crate::synth::Synth;

impl Synth {
    /**
    Returns the number of MIDI channels that the synthesizer uses internally
     */
    pub fn count_midi_channels(&self) -> u8 {
        self.settings.synth.midi_channels
    }

    /**
    Returns the number of audio channels that the synthesizer uses internally
     */
    pub fn count_audio_channels(&self) -> i32 {
        self.settings.synth.audio_channels
    }

    /**
    Returns the number of audio groups that the synthesizer uses internally.
    This is usually identical to audio_channels.
     */
    pub fn count_audio_groups(&self) -> i32 {
        self.settings.synth.audio_groups
    }

    /**
    Returns the number of effects channels that the synthesizer uses internally
     */
    pub fn count_effects_channels(&self) -> i32 {
        self.settings.synth.effects_channels
    }
}
