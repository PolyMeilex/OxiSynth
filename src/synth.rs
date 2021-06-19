mod chorus;
mod count;
mod font;
mod gen;
mod midi;
mod params;
mod reverb;
mod tuning;
mod write;

use crate::{oxi, MidiEvent, OxiError, SettingsError, SynthDescriptor};
pub use tuning::Tuning;

/**
The synth object

You create a new synthesizer with `Synth::new()`.
Use the settings structure to specify the synthesizer characteristics.

You have to load a SoundFont in order to hear any sound.
For that you use the `Synth::sfload()` function.

You can use the audio driver functions described below to open
the audio device and create a background audio thread.

The API for sending MIDI events is probably what you expect:
`Synth::noteon()`, `Synth::noteoff()`, ...
 */
pub struct Synth {
    handle: oxi::synth::Synth,
}

unsafe impl Send for Synth {}

impl Default for Synth {
    fn default() -> Self {
        Self {
            handle: oxi::Synth::default(),
        }
    }
}

impl Synth {
    /**
    Creates a new synthesizer object.

    As soon as the synthesizer is created, it will start playing.
     */
    pub fn new(desc: SynthDescriptor) -> Result<Self, SettingsError> {
        Ok(Synth {
            handle: oxi::synth::Synth::new(desc)?,
        })
    }

    /**
    Set synth sample rate
     */
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.handle.set_sample_rate(sample_rate);
    }

    pub fn send_event(&mut self, event: MidiEvent) -> Result<(), OxiError> {
        self.handle.send_event(event)
    }
}

#[cfg(test)]
mod test {
    use crate::{MidiEvent, SoundFont, Synth, SynthDescriptor};
    use std::{fs::File, io::Write, slice::from_raw_parts};

    #[test]
    fn synth_sf2() {
        let mut pcm = File::create("Boomwhacker.sf2.pcm").unwrap();

        let mut synth = Synth::new(SynthDescriptor::default()).unwrap();

        let mut file = std::fs::File::open("./testdata/Boomwhacker.sf2").unwrap();
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
        pcm.write(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();

        synth
            .send_event(core::MidiEvent::NoteOff {
                channel: 0,
                key: 60,
            })
            .unwrap();

        synth.write(samples.as_mut());
        pcm.write(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();

        drop(synth);
    }
}
