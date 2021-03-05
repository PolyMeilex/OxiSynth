use crate::{engine, Bank, Chan, Prog, Status, Synth};

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
        tuning_bank: Bank,
        tuning_prog: Prog,
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
        tuning_bank: Bank,
        tuning_prog: Prog,
        name: S,
        pitch: &[f64; 12],
    ) -> std::result::Result<(), ()> {
        self.handle
            .create_octave_tuning(tuning_bank as _, tuning_prog as _, name.into(), pitch)
    }

    pub fn activate_octave_tuning<S: Into<String>>(
        &mut self,
        bank: Bank,
        prog: Prog,
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
        tuning_bank: Bank,
        tuning_prog: Prog,
        keys_pitch: KP,
    ) -> std::result::Result<(), ()>
    where
        KP: AsRef<[(u32, f64)]>,
    {
        self.handle
            .tune_notes(tuning_bank, tuning_prog, keys_pitch.as_ref())
    }

    /**
    Select a tuning for a channel.
     */
    pub fn select_tuning(&mut self, chan: Chan, tuning_bank: Bank, tuning_prog: Prog) -> Status {
        Synth::zero_ok(
            self.handle
                .select_tuning(chan as _, tuning_bank as _, tuning_prog as _),
        )
    }

    pub fn activate_tuning(&mut self, chan: Chan, bank: Bank, prog: Prog, apply: bool) -> Status {
        Synth::zero_ok(
            self.handle
                .activate_tuning(chan as _, bank as _, prog as _, apply as _),
        )
    }

    /**
    Set the tuning to the default well-tempered tuning on a channel.
     */
    pub fn reset_tuning(&mut self, chan: Chan) -> std::result::Result<(), ()> {
        self.handle.reset_tuning(chan)
    }

    /**
    Get the iterator throught the list of available tunings.
     */
    pub fn tuning_iter<'a>(&'a mut self) -> impl Iterator<Item = &'a engine::tuning::Tuning> {
        self.handle.tuning_iter()
    }

    /**
    Dump the data of a tuning.

    This function returns both the name and pitch values of a tuning.
     */
    pub fn tuning_dump(
        &self,
        bank: Bank,
        prog: Prog,
    ) -> std::result::Result<(&str, &[f64; 128]), ()> {
        self.handle.tuning_dump(bank, prog)
    }

    /**
    Dump the data of a tuning.

    This function returns the only name of a tuning.
     */
    pub fn tuning_dump_name(&self, bank: Bank, prog: Prog) -> std::result::Result<&str, ()> {
        self.handle.tuning_dump(bank, prog).map(|t| t.0)
    }

    /**
    Dump the data of a tuning.

    This function returns the only pitch values of a tuning.
     */
    pub fn tuning_dump_pitch(
        &self,
        bank: Bank,
        prog: Prog,
    ) -> std::result::Result<&[f64; 128], ()> {
        self.handle.tuning_dump(bank, prog).map(|t| t.1)
    }
}
