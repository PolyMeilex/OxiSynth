use crate::{core::InterpolationMethod, error::OxiError, Synth};

/// Synth settings
impl Synth {
    /// Set synth sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.core.settings.set_sample_rate(sample_rate);
        self.core.voices.set_sample_rate(sample_rate);

        self.core.chorus = oxisynth_chorus::Chorus::new(sample_rate);
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

    /// Set the interpolation method for one channel (`Some(chan)`) or all channels (`None`)
    pub fn set_interpolation_method(
        &mut self,
        chan: Option<usize>,
        interp_method: InterpolationMethod,
    ) {
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

    /// Query the current reverb params
    pub fn reverb_params(&self) -> oxisynth_reverb::ReverbParams {
        self.core.reverb.params()
    }

    /// Set the parameters for the built-in reverb unit
    pub fn set_reverb_params(&mut self, params: &oxisynth_reverb::ReverbParams) {
        self.core.reverb.set_params(params);
    }

    /// Query the current chorus params
    pub fn chorus_params(&self) -> oxisynth_chorus::ChorusParams {
        self.core.chorus.params()
    }

    /// Set up the chorus. It should be turned on with Chorus::set_active().
    /// If faulty parameters are given, all new settings are discarded.
    /// Keep in mind, that the needed CPU time is proportional to `nr`.
    pub fn set_chorus_params(&mut self, params: &oxisynth_chorus::ChorusParams) {
        self.core.chorus.set_params(params);
    }
}
