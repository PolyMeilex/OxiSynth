//! You create a new synthesizer with [`Synth::new()`].
//! Use the settings structure to specify the synthesizer characteristics.
//!
//! You have to load a SoundFont in order to hear any sound.
//! For that you use the [`Synth::add_font()`] function.
//!
//! To send events use: [`Synth::send_event()`] function.
//!
//! To write the results to a buffer use [`Synth::write()`] function.

mod api;
mod arena;
mod core;
mod error;
mod midi_event;

mod unsafe_stuff;

pub use api::Tuning;
pub use core::{GeneratorType, Preset, SoundFont};
pub use error::{OxiError, RangeError, SettingsError};
pub use midi_event::MidiEvent;

#[doc(inline)]
pub use oxisynth_chorus::ChorusParams;
#[doc(inline)]
pub use oxisynth_reverb::ReverbParams;

#[doc(hidden)]
pub use arena::Index;
pub type SoundFontId = Index<SoundFont>;

pub struct SynthDescriptor {
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

impl Default for SynthDescriptor {
    fn default() -> Self {
        Self {
            reverb_active: true,
            chorus_active: true,
            drums_channel_active: true,

            polyphony: 256,
            midi_channels: 16,
            gain: 0.2,
            audio_channels: 1,
            audio_groups: 1,
            sample_rate: 44100.0,
            min_note_length: 10,
        }
    }
}

/// The synth object
///
/// You create a new synthesizer with [`Synth::new()`].
/// Use the settings structure to specify the synthesizer characteristics.
///
/// You have to load a SoundFont in order to hear any sound.
/// For that you use the [`Synth::add_font()`] function.
///
/// To send events use: [`Synth::send_event()`] function.
///
/// To write the results to a buffer use [`Synth::write()`] function.
#[derive(Default)]
pub struct Synth {
    core: crate::core::Core,
}

impl Synth {
    /// Creates a new synthesizer object.
    ///
    /// As soon as the synthesizer is created, it will start playing.
    pub fn new(desc: SynthDescriptor) -> Result<Self, SettingsError> {
        Ok(Synth {
            core: crate::core::Core::new(desc)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{MidiEvent, SoundFont, Synth, SynthDescriptor};
    use std::{fs::File, io::Write};

    #[test]
    fn synth_sf2() {
        let mut pcm = File::create("Boomwhacker.sf2.pcm").unwrap();

        let mut synth = Synth::new(SynthDescriptor::default()).unwrap();

        let mut file = std::fs::File::open("../testdata/Boomwhacker.sf2").unwrap();
        let font = SoundFont::load(&mut file).unwrap();

        synth.add_font(font, true);

        let mut samples = [0f32; 44100 * 2];

        synth
            .send_event(MidiEvent::NoteOn {
                channel: 0,
                key: 60,
                vel: 127,
            })
            .unwrap();

        synth.write(samples.as_mut());
        pcm.write_all(crate::unsafe_stuff::slice_f32_to_u8(&samples))
            .unwrap();

        synth
            .send_event(MidiEvent::NoteOff {
                channel: 0,
                key: 60,
            })
            .unwrap();

        synth.write(samples.as_mut());
        pcm.write_all(crate::unsafe_stuff::slice_f32_to_u8(&samples))
            .unwrap();

        drop(synth);
    }
}
