use std::sync::Arc;

use crate::error::OxiError;
use crate::{Preset, Settings, Synth};

use crate::core::InterpolationMethod;

// Synthesis parameters
impl Synth {
    /// Get a reference to the settings of the synthesizer.
    pub fn settings(&self) -> &Settings {
        &self.core.settings
    }

    /// Set the master gain
    pub fn set_gain(&mut self, gain: f32) {
        let gain = gain.clamp(0.0, 10.0);
        self.core.settings.gain = gain;
        self.core.voices.set_gain(gain)
    }

    /// Get the master gain
    pub fn gain(&self) -> f32 {
        self.core.settings.gain
    }

    /// Set the polyphony limit
    pub fn set_polyphony(&mut self, polyphony: u16) -> Result<(), OxiError> {
        if polyphony < 1 {
            return Err(OxiError::InvalidPolyphony);
        }

        self.core.settings.polyphony = polyphony;
        self.core.voices.set_polyphony_limit(polyphony as usize);

        Ok(())
    }

    /// Get the polyphony limit (FluidSynth >= 1.0.6)
    pub fn polyphony(&self) -> u32 {
        self.core.settings.polyphony as u32
    }

    /// Get the internal buffer size. The internal buffer size if not the
    /// same thing as the buffer size specified in the
    /// settings. Internally, the synth *always* uses a specific buffer
    /// size independent of the buffer size used by the audio driver. The
    /// internal buffer size is normally 64 samples. The reason why it
    /// uses an internal buffer size is to allow audio drivers to call the
    /// synthesizer with a variable buffer length. The internal buffer
    /// size is useful for client who want to optimize their buffer sizes.
    pub fn internal_buffer_size(&self) -> usize {
        64
    }

    /// Set the interpolation method for one channel (`Some(chan)`) or all channels (`None`)
    pub fn set_interp_method(&mut self, chan: Option<usize>, interp_method: InterpolationMethod) {
        if let Some(chan) = chan {
            let ch = self.core.channels.iter_mut().find(|ch| ch.id() == chan);

            if let Some(ch) = ch {
                ch.set_interp_method(interp_method);
            }
        } else {
            for ch in self.core.channels.iter_mut() {
                ch.set_interp_method(interp_method);
            }
        }
    }

    pub fn channel_preset(&self, chan: u8) -> Option<&Arc<Preset>> {
        if let Ok(channel) = self.core.channels.get(chan as usize) {
            channel.preset()
        } else {
            log::warn!("Channel out of range");
            None
        }
    }
}
