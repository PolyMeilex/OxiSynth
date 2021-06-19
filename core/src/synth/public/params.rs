use std::rc::Rc;

use crate::settings::Settings;
use crate::synth::{Chorus, InterpolationMethod, Preset, Synth};

impl Synth {
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.settings.sample_rate = sample_rate;
        self.voices.set_sample_rate(sample_rate);

        self.chorus = Chorus::new(sample_rate, self.chorus.active());
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /**
    Set the master gain
     */
    pub fn set_gain(&mut self, gain: f32) {
        self.settings.gain = if gain < 0.0 {
            0.0
        } else if gain > 10.0 {
            10.0
        } else {
            gain
        };

        self.voices.set_gain(gain)
    }

    /**
    Get the master gain
     */
    pub fn gain(&self) -> f32 {
        self.settings.gain
    }

    /**
    Set the polyphony limit
     */
    pub fn set_polyphony(&mut self, polyphony: u16) -> Result<(), ()> {
        if polyphony < 1 {
            Err(())
        } else {
            self.settings.polyphony = polyphony;
            self.voices.set_polyphony_limit(polyphony as usize);

            Ok(())
        }
    }

    /**
    Get the polyphony limit (FluidSynth >= 1.0.6)
     */
    pub fn polyphony(&self) -> u32 {
        self.settings.polyphony as u32
    }

    /**
    Get the internal buffer size. The internal buffer size if not the
    same thing as the buffer size specified in the
    settings. Internally, the synth *always* uses a specific buffer
    size independent of the buffer size used by the audio driver. The
    internal buffer size is normally 64 samples. The reason why it
    uses an internal buffer size is to allow audio drivers to call the
    synthesizer with a variable buffer length. The internal buffer
    size is useful for client who want to optimize their buffer sizes.
     */
    pub fn internal_bufsize(&self) -> usize {
        64
    }

    /**
     * Set the interpolation method for one channel (`Some(chan)`) or all channels (`None`)
     */
    pub fn set_interp_method(&mut self, chan: Option<usize>, interp_method: InterpolationMethod) {
        if let Some(chan) = chan {
            let ch = self.channels.iter_mut().find(|ch| ch.id() == chan);

            if let Some(ch) = ch {
                ch.set_interp_method(interp_method);
            }
        } else {
            for ch in self.channels.iter_mut() {
                ch.set_interp_method(interp_method);
            }
        }
    }

    pub fn channel_preset(&self, chan: u8) -> Option<&Rc<Preset>> {
        if let Ok(channel) = self.channels.get(chan as usize) {
            channel.preset()
        } else {
            log::warn!("Channel out of range");
            None
        }
    }
}
