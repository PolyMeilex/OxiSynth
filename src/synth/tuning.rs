use crate::{oxi, Synth};
pub use oxi::tuning::{Tuning, TuningManager};

/**
 * Tuning
 */
impl Synth {
    /// Select a tuning for a channel.
    pub fn channel_set_tuning(&mut self, chan: u8, tuning: Tuning) -> Result<(), &str> {
        self.handle.channel_set_tuning(chan, tuning)
    }

    /// Set the tuning to the default well-tempered tuning on a channel.
    pub fn channel_reset_tuning(&mut self, chan: u8) -> Result<(), &str> {
        self.handle.channel_reset_tuning(chan)
    }
}
