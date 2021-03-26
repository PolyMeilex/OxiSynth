#[derive(Clone, Copy)]
pub struct Tuning {
    pub(crate) bank: u32,
    pub(crate) program: u32,
    pub(crate) pitch: [f64; 128],
}

impl Tuning {
    pub fn new(bank: u32, program: u32) -> Self {
        let mut tuning = Self {
            bank,
            program,
            pitch: [0.0; 128],
        };
        for i in 0..128 {
            tuning.pitch[i] = i as f64 * 100.0;
        }
        return tuning;
    }

    /**
    Create a new key-based tuning with given name, number, and
    pitches. The array 'pitches' should have length 128 and contains
    the pitch in cents of every key in cents. However, if 'pitches' is
    NULL, a new tuning is created with the well-tempered scale.
     */

    pub fn new_key_tuning(bank: u32, prog: u32, pitch: &[f64; 128]) -> Self {
        let mut tuning = Self::new(bank, prog);
        tuning.set_all(pitch);
        tuning
    }

    /**
    Create a new octave-based tuning with given name, number, and
    pitches.  The array 'pitches' should have length 12 and contains
    derivation in cents from the well-tempered scale. For example, if
    pitches[0] equals -33, then the C-keys will be tuned 33 cents
    below the well-tempered C.
     */
    pub fn new_octave_tuning(bank: u32, prog: u32, pitch: &[f64; 12]) -> Self {
        let mut tuning = Self::new(bank, prog);
        tuning.set_octave(pitch);
        tuning
    }
}

impl Tuning {
    pub fn set_octave(&mut self, pitch_deriv: &[f64; 12]) {
        for i in 0..128 {
            self.pitch[i] = i as f64 * 100.0 + pitch_deriv[i % 12];
        }
    }

    pub fn set_all(&mut self, pitch: &[f64; 128]) {
        for i in 0..128 {
            self.pitch[i] = pitch[i];
        }
    }

    pub fn set_pitch(&mut self, key: u32, pitch: f64) {
        if key < 128 {
            self.pitch[key as usize] = pitch;
        }
    }

    pub fn tune_notes(&mut self, key_pitch: &[(u32, f64)]) -> Result<(), ()> {
        for (key, pitch) in key_pitch.iter() {
            self.set_pitch(*key, *pitch);
        }
        Ok(())
    }
}
