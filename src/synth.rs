mod chorus;
mod count;
mod font;
mod gen;
mod loader;
mod midi;
mod misc;
mod params;
mod reverb;
mod tuning;
mod write;

pub use self::tuning::TuningIter;
pub use self::write::IsSamples;

use crate::{engine, Error, Result};

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
    handle: engine::synth::Synth,
}

unsafe impl Send for Synth {}

impl Synth {
    /**
    Creates a new synthesizer object.

    As soon as the synthesizer is created, it will start playing.
     */
    pub fn new(settings: engine::settings::new::Settings) -> Result<Self> {
        match engine::synth::Synth::new(settings) {
            Ok(handle) => return Ok(Synth { handle }),
            Err(_) => return Err(Error::Alloc),
        }
    }

    /**
    Set synth sample rate
     */
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        unsafe {
            self.handle.set_sample_rate(sample_rate);
        }
    }

    // /**
    // Get a reference to the settings of the synthesizer.
    //  */
    // pub fn get_settings(&mut self) -> SettingsRef<'_> {
    //     SettingsRef::from_ptr(&mut self.handle.old_settings)
    // }
}

#[cfg(test)]
mod test {
    use super::{engine::settings::new::Settings, Synth};
    use std::{fs::File, io::Write, slice::from_raw_parts};

    #[test]
    fn synth_sf2() {
        let mut pcm = File::create("Boomwhacker.sf2.pcm").unwrap();

        let settings = Settings::default();

        let mut synth = Synth::new(settings).unwrap();

        synth.sfload("./testdata/Boomwhacker.sf2", true).unwrap();

        let mut samples = [0f32; 44100 * 2];

        synth.note_on(0, 60, 127).unwrap();

        synth.write(samples.as_mut()).unwrap();
        pcm.write(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();

        synth.note_off(0, 60).unwrap();

        synth.write(samples.as_mut()).unwrap();
        pcm.write(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();

        drop(synth);
    }
}
