use crate::{oxi, Synth};
pub use oxi::tuning::Tuning;

/**
 * Tuning
 */
impl Synth {
    /// Select a tuning for a channel.
    pub fn channel_select_tuning(&mut self, chan: u8, bank: u32, prog: u32) -> Result<(), &str> {
        self.handle.channel_select_tuning(chan, bank, prog)
    }

    /// Set the tuning to the default well-tempered tuning on a channel.
    pub fn channel_reset_tuning(&mut self, chan: u8) -> Result<(), &str> {
        self.handle.channel_reset_tuning(chan)
    }

    /// Adds tuning to synth.
    ///
    /// If tuning with the same bank and program already exsists it gets replaced.
    pub fn add_tuning(&mut self, tuning: Tuning) -> Result<(), &str> {
        self.handle.add_tuning(tuning)
    }

    // Removes tuning asignet to specified bank and program
    pub fn remove_tuning(&mut self, bank: u32, program: u32) -> Result<Tuning, &str> {
        self.handle.remove_tuning(bank, program)
    }

    // Gets tuning asignet to specified bank and program
    pub fn get_tuning(&self, bank: u32, program: u32) -> Option<&Tuning> {
        self.handle.get_tuning(bank, program)
    }

    // Gets tuning asignet to specified bank and program
    pub fn get_tuning_mut(&mut self, bank: u32, program: u32) -> Option<&mut Tuning> {
        self.handle.get_tuning_mut(bank, program)
    }

    pub fn tuning_iter<'a>(&'a self) -> impl Iterator<Item = &'a Tuning> {
        self.handle.tuning_iter()
    }

    pub fn tuning_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Tuning> {
        self.handle.tuning_iter_mut()
    }
}
