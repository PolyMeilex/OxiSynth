use crate::oxi::reverb::Reverb;
use crate::Synth;

impl Synth {
    pub fn get_reverb(&self) -> &Reverb {
        &self.handle.reverb
    }

    pub fn get_reverb_mut(&mut self) -> &mut Reverb {
        &mut self.handle.reverb
    }
}
