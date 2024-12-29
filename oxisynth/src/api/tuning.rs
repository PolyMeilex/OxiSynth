use crate::OxiError;

/// Tuning
impl crate::Synth {
    /// Select a tuning for a channel.
    pub fn set_tuning(&mut self, chan: u8, tuning: Option<Tuning>) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(chan as usize)?;
        channel.set_tuning(tuning);
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Tuning {
    pub(crate) pitch: [f64; 128],
}

impl Default for Tuning {
    fn default() -> Self {
        Self::new()
    }
}

impl Tuning {
    pub fn new() -> Self {
        Self {
            pitch: std::array::from_fn(|i| i as f64 * 100.0),
        }
    }

    /// Create a new key-based tuning with given pitches.
    /// The array should contain the pitch of every key in cents.
    pub fn new_key_tuning(pitch: [f64; 128]) -> Self {
        Self { pitch }
    }

    /// Create a new octave-based tuning with given pitches.
    /// The array should contains derivation in cents from the well-tempered scale.
    ///
    /// For example, if pitches[0] equals -33, then the C-keys will be tuned 33 cents
    /// below the well-tempered C.
    pub fn new_octave_tuning(pitch: &[f64; 12]) -> Self {
        let mut tuning = Self::new();

        for (i, v) in tuning.pitch.iter_mut().enumerate() {
            *v = i as f64 * 100.0 + pitch[i % 12];
        }

        tuning
    }

    pub fn as_slice(&self) -> &[f64; 128] {
        &self.pitch
    }

    pub fn as_slice_mut(&mut self) -> &mut [f64; 128] {
        &mut self.pitch
    }
}
