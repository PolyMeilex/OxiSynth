use crate::synth::Synth;
use crate::tuning::Tuning;

impl Synth {
    /// Select a tuning for a channel.
    pub fn channel_select_tuning(&mut self, chan: u8, bank: u32, prog: u32) -> Result<(), &str> {
        if let Some(tuning) = self.get_tuning(bank, prog).map(|t| t.clone()) {
            if let Some(channel) = self.channels.get_mut(chan as usize) {
                channel.tuning = Some(tuning);
                Ok(())
            } else {
                Err("Channel out of range")
            }
        } else {
            Err("No Tuning found")
        }
    }

    /// Set the tuning to the default well-tempered tuning on a channel.
    pub fn channel_reset_tuning(&mut self, chan: u8) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            channel.tuning = None;
            Ok(())
        } else {
            Err("channel_select_tuning")
        }
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
    pub fn get_tuning(&self, bank: u32, program: u32) -> Option<&Tuning> {
        let bank = bank as usize;
        let program = program as usize;

        self.tuning
            .get(bank)
            .and_then(|bank| bank.get(program).and_then(|t| t.as_ref()))
    }

    // Gets tuning asignet to specified bank and program
    pub fn get_tuning_mut(&mut self, bank: u32, program: u32) -> Option<&mut Tuning> {
        let bank = bank as usize;
        let program = program as usize;

        self.tuning
            .get_mut(bank)
            .and_then(|bank| bank.get_mut(program).and_then(|t| t.as_mut()))
    }

    pub fn tuning_iter<'a>(&'a self) -> impl Iterator<Item = &'a Tuning> {
        self.tuning.iter().flatten().filter_map(|t| t.as_ref())
    }

    pub fn tuning_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Tuning> {
        self.tuning.iter_mut().flatten().filter_map(|t| t.as_mut())
    }
}

#[cfg(test)]
mod test {
    use crate::{Synth, Tuning};

    #[test]
    fn tuning() {
        let mut synth = Synth::default();

        // Out of range test:
        synth.get_tuning(120, 120);
        synth.get_tuning(999, 999);

        // Adding a tunning
        let (bank, program) = (15, 2);

        let tuning = Tuning::new(bank, program);
        synth.add_tuning(tuning).unwrap();

        let tuning = synth.get_tuning(bank, program).unwrap();
        assert_eq!(tuning.bank, bank);
        assert_eq!(tuning.program, program);

        synth.get_tuning_mut(bank, program).unwrap();
    }
}
