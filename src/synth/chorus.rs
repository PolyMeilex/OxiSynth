use crate::{oxi, Synth};

use oxi::chorus::Chorus;

impl Synth {
    pub fn chorus(&self) -> &Chorus {
        &self.handle.chorus
    }

    pub fn chorus_mut(&mut self) -> &mut Chorus {
        &mut self.handle.chorus
    }
}
