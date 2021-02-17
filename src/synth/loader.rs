use crate::{Loader, Synth};

impl Synth {
    /**
    Add a SoundFont loader to the synthesizer. Note that SoundFont
    loader don't necessarily load SoundFonts. They can load any type
    of wavetable data but export a SoundFont interface.
     */
    pub fn add_sfloader(&mut self, loader: Loader) {
        self.handle.add_sfloader(loader.into_ptr());
    }
}
