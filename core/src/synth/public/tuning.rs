use crate::synth::Synth;
use crate::tuning::Tuning;

impl Synth {
    /// Select a tuning for a channel.
    pub fn channel_set_tuning(&mut self, chan: u8, tuning: Tuning) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            channel.set_tuning(Some(tuning));
            Ok(())
        } else {
            Err("Channel out of range")
        }
    }

    /// Set the tuning to the default well-tempered tuning on a channel.
    pub fn channel_reset_tuning(&mut self, chan: u8) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            channel.set_tuning(None);
            Ok(())
        } else {
            Err("channel_select_tuning")
        }
    }
}
