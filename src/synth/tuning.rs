use crate::{oxi, Synth};
use oxi::tuning::Tuning;

/**
 * Tuning
 */
impl Synth {
    /**
    Create a new key-based tuning with given name, number, and
    pitches. The array 'pitches' should have length 128 and contains
    the pitch in cents of every key in cents. However, if 'pitches' is
    NULL, a new tuning is created with the well-tempered scale.
     */
    pub fn create_key_tuning<S: Into<String>>(
        &mut self,
        tuning_bank: u32,
        tuning_prog: u32,
        name: S,
        pitch: &[f64; 128],
    ) -> std::result::Result<(), ()> {
        self.handle
            .create_key_tuning(tuning_bank as _, tuning_prog as _, name.into(), pitch)
    }

    /**
    Create a new octave-based tuning with given name, number, and
    pitches.  The array 'pitches' should have length 12 and contains
    derivation in cents from the well-tempered scale. For example, if
    pitches[0] equals -33, then the C-keys will be tuned 33 cents
    below the well-tempered C.
     */
    pub fn create_octave_tuning<S: Into<String>>(
        &mut self,
        tuning_bank: u32,
        tuning_prog: u32,
        name: S,
        pitch: &[f64; 12],
    ) -> std::result::Result<(), ()> {
        self.handle
            .create_octave_tuning(tuning_bank as _, tuning_prog as _, name.into(), pitch)
    }

    pub fn activate_octave_tuning<S: Into<String>>(
        &mut self,
        bank: u32,
        prog: u32,
        name: S,
        pitch: &[f64; 12],
    ) -> std::result::Result<(), ()> {
        self.handle
            .activate_octave_tuning(bank as _, prog as _, name.into(), pitch)
    }

    /**
    Request a note tuning changes. Both they 'keys' and 'pitches'
    arrays should be of length 'num_pitches'. If 'apply' is non-zero,
    the changes should be applied in real-time, i.e. sounding notes
    will have their pitch updated. Changes will be available for newly triggered notes only.
     */
    pub fn tune_notes<KP>(
        &mut self,
        tuning_bank: u32,
        tuning_prog: u32,
        keys_pitch: KP,
    ) -> Result<(), ()>
    where
        KP: AsRef<[(u32, f64)]>,
    {
        self.handle
            .tune_notes(tuning_bank, tuning_prog, keys_pitch.as_ref())
    }

    /**
    Select a tuning for a channel.
     */
    pub fn select_tuning(
        &mut self,
        chan: u8,
        tuning_bank: u32,
        tuning_prog: u32,
    ) -> Result<(), ()> {
        self.handle.select_tuning(chan, tuning_bank, tuning_prog)
    }

    pub fn activate_tuning(&mut self, chan: u8, bank: u32, prog: u32) -> Result<(), ()> {
        self.handle.activate_tuning(chan, bank, prog)
    }

    /**
    Set the tuning to the default well-tempered tuning on a channel.
     */
    pub fn reset_tuning(&mut self, chan: u8) -> std::result::Result<(), ()> {
        self.handle.reset_tuning(chan)
    }

    /**
    Get the iterator throught the list of available tunings.
     */
    pub fn tuning_iter<'a>(&'a mut self) -> impl Iterator<Item = &'a Tuning> {
        self.handle.tuning_iter()
    }

    /**
    Dump the data of a tuning.

    This function returns both the name and pitch values of a tuning.
     */
    pub fn tuning_dump(
        &self,
        bank: u32,
        prog: u32,
    ) -> std::result::Result<(&str, &[f64; 128]), ()> {
        self.handle.tuning_dump(bank, prog)
    }

    /**
    Dump the data of a tuning.

    This function returns the only name of a tuning.
     */
    pub fn tuning_dump_name(&self, bank: u32, prog: u32) -> Result<&str, ()> {
        self.handle.tuning_dump(bank, prog).map(|t| t.0)
    }

    /**
    Dump the data of a tuning.

    This function returns the only pitch values of a tuning.
     */
    pub fn tuning_dump_pitch(&self, bank: u32, prog: u32) -> Result<&[f64; 128], ()> {
        self.handle.tuning_dump(bank, prog).map(|t| t.1)
    }
}
