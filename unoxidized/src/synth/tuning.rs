use crate::synth::Synth;
use crate::tuning::Tuning;

impl Synth {
    /**
    Create a new key-based tuning with given name, number, and
    pitches. The array 'pitches' should have length 128 and contains
    the pitch in cents of every key in cents. However, if 'pitches' is
    NULL, a new tuning is created with the well-tempered scale.
     */

    pub fn create_key_tuning(
        &mut self,
        bank: u32,
        prog: u32,
        name: String,
        pitch: &[f64; 128],
    ) -> Result<(), ()> {
        let tuning = self.create_tuning(bank, prog, name)?;
        tuning.set_all(pitch);
        Ok(())
    }

    /**
    Create a new octave-based tuning with given name, number, and
    pitches.  The array 'pitches' should have length 12 and contains
    derivation in cents from the well-tempered scale. For example, if
    pitches[0] equals -33, then the C-keys will be tuned 33 cents
    below the well-tempered C.
     */
    pub fn create_octave_tuning(
        &mut self,
        bank: u32,
        prog: u32,
        name: String,
        pitch: &[f64; 12],
    ) -> Result<(), ()> {
        if !(bank < 128) {
            Err(())
        } else if !(prog < 128) {
            Err(())
        } else {
            let tuning = self.create_tuning(bank, prog, name)?;

            tuning.set_octave(pitch);
            Ok(())
        }
    }

    pub fn activate_octave_tuning(
        &mut self,
        bank: u32,
        prog: u32,
        name: String,
        pitch: &[f64; 12],
    ) -> Result<(), ()> {
        self.create_octave_tuning(bank, prog, name, pitch)
    }

    /**
    Request a note tuning changes. Both they 'keys' and 'pitches'
    arrays should be of length 'num_pitches'. If 'apply' is non-zero,
    the changes should be applied in real-time, i.e. sounding notes
    will have their pitch updated. 'APPLY' IS CURRENTLY IGNORED. The
    changes will be available for newly triggered notes only.
     */
    pub fn tune_notes(&mut self, bank: u32, prog: u32, key_pitch: &[(u32, f64)]) -> Result<(), ()> {
        if !(bank < 128) {
            Err(())
        } else if !(prog < 128) {
            Err(())
        } else {
            let tuning = self.create_tuning(bank, prog, "Unnamed".into())?;
            for (key, pitch) in key_pitch.iter() {
                tuning.set_pitch(*key, *pitch);
            }
            Ok(())
        }
    }

    /**
    Select a tuning for a channel.
     */
    pub fn select_tuning(&mut self, chan: u8, bank: u32, prog: u32) -> Result<(), ()> {
        let tuning;
        if !(bank < 128) {
            return Err(());
        }
        if !(prog < 128) {
            return Err(());
        }
        tuning = self.get_tuning(bank, prog);
        if tuning.is_none() {
            return Err(());
        }
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return Err(());
        }
        self.channel[chan as usize].tuning = Some(tuning.unwrap().clone());
        Ok(())
    }

    pub fn activate_tuning(&mut self, chan: u8, bank: u32, prog: u32) -> Result<(), ()> {
        self.select_tuning(chan, bank, prog)
    }

    /**
    Set the tuning to the default well-tempered tuning on a channel.
     */
    pub fn reset_tuning(&mut self, chan: u8) -> Result<(), ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range");
            Err(())
        } else {
            self.channel[chan as usize].tuning = None;
            Ok(())
        }
    }

    pub fn tuning_iter<'a>(&'a mut self) -> impl Iterator<Item = &'a Tuning> {
        self.tuning
            .iter()
            .flatten()
            .filter_map(|t| if let Some(t) = t { Some(t) } else { None })
    }

    pub fn tuning_dump(&self, bank: u32, prog: u32) -> Result<(&str, &[f64; 128]), ()> {
        match self.get_tuning(bank, prog) {
            Some(tuning) => Ok((tuning.get_name(), &tuning.pitch)),
            None => Err(()),
        }
    }

    fn get_tuning(&self, bank: u32, prog: u32) -> Option<&Tuning> {
        if bank >= 128 {
            log::warn!("Bank number out of range");
            None
        } else if prog >= 128 {
            log::warn!("Program number out of range");
            None
        } else {
            self.tuning[bank as usize][prog as usize].as_ref()
        }
    }

    fn create_tuning<'a>(
        &'a mut self,
        bank: u32,
        prog: u32,
        name: String,
    ) -> Result<&'a mut Tuning, ()> {
        if bank >= 128 {
            log::warn!("Bank number out of range",);
            Err(())
        } else if prog >= 128 {
            log::warn!("Program number out of range",);
            Err(())
        } else {
            let tuning = self.tuning[bank as usize][prog as usize]
                .get_or_insert_with(|| Tuning::new(name.clone(), bank, prog));

            tuning.set_name(name);

            Ok(tuning)
        }
    }
}
