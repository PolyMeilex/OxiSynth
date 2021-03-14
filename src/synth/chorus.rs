use crate::{oxi, Synth};

use oxi::chorus::Chorus;

impl Synth {
    pub fn get_chorus(&self) -> &Chorus {
        &self.handle.chorus
    }

    pub fn get_chorus_mut(&mut self) -> &mut Chorus {
        &mut self.handle.chorus
    }
}
