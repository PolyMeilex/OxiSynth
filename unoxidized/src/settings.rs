pub struct SynthSettings {
    pub dump: bool,

    pub reverb_active: bool,
    pub chorus_active: bool,
    pub ladspa_active: bool,
    pub drums_channel_active: bool,

    /// Def: 256
    /// Min: 16
    /// Max: 4096
    pub polyphony: u16,
    /// Def: 16
    /// Min: 16
    /// Max: 256
    pub midi_channels: u8,
    /// Def: 0.2
    /// Min: 0.0
    /// Max: 10.0
    pub gain: f64,
    /// Def: 1
    /// Min: 1
    /// Max: 256
    pub audio_channels: i32,
    /// Def: 1
    /// Min: 1
    /// Max: 256
    pub audio_groups: i32,
    /// Def: 2
    /// Min: 2
    /// Max: 2
    pub effects_channels: i32,
    /// Def: 44100.0
    /// Min: 22050.0
    /// Max: 96000.0
    pub sample_rate: f32,
    /// Def: 10
    /// Min: 0
    /// Max: 65535
    pub min_note_length: i32,
}

impl Default for SynthSettings {
    fn default() -> Self {
        Self {
            dump: false,

            reverb_active: true,
            chorus_active: true,
            ladspa_active: false,
            drums_channel_active: true,

            polyphony: 256,
            midi_channels: 16,
            gain: 0.2,
            audio_channels: 1,
            audio_groups: 1,
            effects_channels: 2,
            sample_rate: 44100.0,
            min_note_length: 10,
        }
    }
}

#[derive(Default)]
pub struct MidiSettings {
    pub portname: String,
}

#[derive(Default)]
pub struct Settings {
    pub synth: SynthSettings,
    pub midi: MidiSettings,
}
