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
        tuning
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
        self.pitch = *pitch;
    }

    pub fn set_pitch(&mut self, key: u32, pitch: f64) {
        if key < 128 {
            self.pitch[key as usize] = pitch;
        }
    }

    pub fn tune_notes(&mut self, key_pitch: &[(u32, f64)]) {
        for (key, pitch) in key_pitch.iter() {
            self.set_pitch(*key, *pitch);
        }
    }
}

pub struct TuningManager {
    tuning: Vec<Vec<Option<Tuning>>>,
}

impl Default for TuningManager {
    fn default() -> Self {
        Self {
            tuning: vec![vec![None; 128]; 128],
        }
    }
}

impl TuningManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds tuning to synth.
    ///
    /// If tuning with the same bank and program already exsists it gets replaced.
    pub fn add_tuning(&mut self, tuning: Tuning) -> Result<(), &str> {
        let bank = tuning.bank as usize;
        let program = tuning.program as usize;

        if let Some(bank) = self.tuning.get_mut(bank) {
            if let Some(t) = bank.get_mut(program) {
                *t = Some(tuning);
                Ok(())
            } else {
                Err("Program number out of range")
            }
        } else {
            Err("Bank number out of range")
        }
    }

    // Removes tuning asignet to specified bank and program
    pub fn remove_tuning(&mut self, bank: u32, program: u32) -> Result<Tuning, &str> {
        let bank = bank as usize;
        let program = program as usize;

        if let Some(bank) = self.tuning.get_mut(bank) {
            if let Some(t) = bank.get_mut(program) {
                let t = t.take();
                t.ok_or("No tuning found")
            } else {
                Err("Program number out of range")
            }
        } else {
            Err("Bank number out of range")
        }
    }

    // Gets tuning asignet to specified bank and program
    pub fn tuning(&self, bank: u32, program: u32) -> Option<&Tuning> {
        let bank = bank as usize;
        let program = program as usize;

        self.tuning
            .get(bank)
            .and_then(|bank| bank.get(program).and_then(|t| t.as_ref()))
    }

    // Gets tuning asignet to specified bank and program
    pub fn tuning_mut(&mut self, bank: u32, program: u32) -> Option<&mut Tuning> {
        let bank = bank as usize;
        let program = program as usize;

        self.tuning
            .get_mut(bank)
            .and_then(|bank| bank.get_mut(program).and_then(|t| t.as_mut()))
    }

    pub fn tuning_iter(&self) -> impl Iterator<Item = &Tuning> {
        self.tuning.iter().flatten().filter_map(|t| t.as_ref())
    }

    pub fn tuning_iter_mut(&mut self) -> impl Iterator<Item = &mut Tuning> {
        self.tuning.iter_mut().flatten().filter_map(|t| t.as_mut())
    }
}

#[cfg(test)]
mod test {
    use super::{Tuning, TuningManager};

    #[test]
    fn tuning() {
        let mut tuning_manager = TuningManager::new();

        assert!(tuning_manager.tuning(120, 120).is_none());
        assert!(tuning_manager.tuning(999, 999).is_none());

        // Adding a tunning
        let (bank, program) = (15, 2);

        let tuning = Tuning::new(bank, program);
        tuning_manager.add_tuning(tuning).unwrap();

        let tuning = tuning_manager.tuning(bank, program).unwrap();
        assert_eq!(tuning.bank, bank);
        assert_eq!(tuning.program, program);

        tuning_manager.tuning_mut(bank, program).unwrap();
    }
}
