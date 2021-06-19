use crate::synth::Synth;
use crate::tuning::Tuning;
use crate::OxiError;

impl Synth {
    /// Select a tuning for a channel.
    pub fn channel_set_tuning(&mut self, chan: u8, tuning: Tuning) -> Result<(), OxiError> {
        let channel = self.channels.get_mut(chan as usize)?;
        channel.set_tuning(Some(tuning));
        Ok(())
    }

    /// Set the tuning to the default well-tempered tuning on a channel.
    pub fn channel_reset_tuning(&mut self, chan: u8) -> Result<(), OxiError> {
        let channel = self.channels.get_mut(chan as usize)?;
        channel.set_tuning(None);
        Ok(())
    }
}
