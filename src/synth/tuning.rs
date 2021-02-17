use crate::{engine, Bank, Chan, Prog, Result, Status, Synth};
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::null_mut,
};

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
    pub fn create_key_tuning<S: AsRef<str>>(
        &mut self,
        tuning_bank: Bank,
        tuning_prog: Prog,
        name: S,
        pitch: &[f64; 128],
    ) -> Status {
        let name = CString::new(name.as_ref()).unwrap();
        Synth::zero_ok(unsafe {
            self.handle.create_key_tuning(
                tuning_bank as _,
                tuning_prog as _,
                name.as_bytes_with_nul(),
                pitch,
            )
        })
    }

    /**
    Create a new octave-based tuning with given name, number, and
    pitches.  The array 'pitches' should have length 12 and contains
    derivation in cents from the well-tempered scale. For example, if
    pitches[0] equals -33, then the C-keys will be tuned 33 cents
    below the well-tempered C.
     */
    pub fn create_octave_tuning<S: AsRef<str>>(
        &mut self,
        tuning_bank: Bank,
        tuning_prog: Prog,
        name: S,
        pitch: &[f64; 12],
    ) -> Status {
        let name = CString::new(name.as_ref()).unwrap();
        Synth::zero_ok(unsafe {
            self.handle.create_octave_tuning(
                tuning_bank as _,
                tuning_prog as _,
                name.as_bytes_with_nul(),
                pitch,
            )
        })
    }

    pub fn activate_octave_tuning<S: AsRef<str>>(
        &mut self,
        bank: Bank,
        prog: Prog,
        name: S,
        pitch: &[f64; 12],
        apply: bool,
    ) -> Status {
        let name = CString::new(name.as_ref()).unwrap();
        Synth::zero_ok(unsafe {
            self.handle.activate_octave_tuning(
                bank as _,
                prog as _,
                name.as_bytes_with_nul(),
                pitch,
                apply as _,
            )
        })
    }

    /**
    Request a note tuning changes. Both they 'keys' and 'pitches'
    arrays should be of length 'num_pitches'. If 'apply' is non-zero,
    the changes should be applied in real-time, i.e. sounding notes
    will have their pitch updated. 'APPLY' IS CURRENTLY IGNORED. The
    changes will be available for newly triggered notes only.
     */
    pub fn tune_notes<K, P>(
        &mut self,
        tuning_bank: Bank,
        tuning_prog: Prog,
        keys: K,
        pitch: P,
        apply: bool,
    ) -> Status
    where
        K: AsRef<[u32]>,
        P: AsRef<[f64]>,
    {
        let keys = keys.as_ref();
        let pitch = pitch.as_ref();
        let len = keys.len().min(pitch.len());
        Synth::zero_ok(unsafe {
            self.handle.tune_notes(
                tuning_bank as _,
                tuning_prog as _,
                len as _,
                keys.as_ptr() as _,
                pitch.as_ptr() as _,
                apply as _,
            )
        })
    }

    /**
    Select a tuning for a channel.
     */
    pub fn select_tuning(&mut self, chan: Chan, tuning_bank: Bank, tuning_prog: Prog) -> Status {
        Synth::zero_ok(unsafe {
            self.handle
                .select_tuning(chan as _, tuning_bank as _, tuning_prog as _)
        })
    }

    pub fn activate_tuning(&mut self, chan: Chan, bank: Bank, prog: Prog, apply: bool) -> Status {
        Synth::zero_ok(unsafe {
            self.handle
                .activate_tuning(chan as _, bank as _, prog as _, apply as _)
        })
    }

    /**
    Set the tuning to the default well-tempered tuning on a channel.
     */
    pub fn reset_tuning(&mut self, chan: Chan) -> Status {
        Synth::zero_ok(unsafe { self.handle.reset_tuning(chan as _) })
    }

    /**
    Get the iterator throught the list of available tunings.
     */
    pub fn tuning_iter(&mut self) -> TuningIter<'_> {
        TuningIter::from_ptr(&mut self.handle)
    }

    /**
    Dump the data of a tuning.

    This function returns both the name and pitch values of a tuning.
     */
    pub fn tuning_dump(&self, bank: Bank, prog: Prog) -> Result<(String, [f64; 128])> {
        const NAME_LEN: usize = 128;

        let mut name = MaybeUninit::<[u8; NAME_LEN]>::uninit();
        let mut pitch = MaybeUninit::<[f64; 128]>::uninit();

        Synth::zero_ok(unsafe {
            self.handle.tuning_dump(
                bank as _,
                prog as _,
                name.as_mut_ptr() as _,
                NAME_LEN as _,
                pitch.as_mut_ptr() as _,
            )
        })?;
        Ok((
            (unsafe { CStr::from_ptr(name.as_ptr() as _) })
                .to_str()
                .unwrap()
                .into(),
            unsafe { pitch.assume_init() },
        ))
    }

    /**
    Dump the data of a tuning.

    This function returns the only name of a tuning.
     */
    pub fn tuning_dump_name(&self, bank: Bank, prog: Prog) -> Result<String> {
        const NAME_LEN: usize = 128;

        let mut name = MaybeUninit::<[u8; NAME_LEN]>::uninit();

        Synth::zero_ok(unsafe {
            self.handle.tuning_dump(
                bank as _,
                prog as _,
                name.as_mut_ptr() as _,
                NAME_LEN as _,
                null_mut(),
            )
        })?;
        Ok((unsafe { CStr::from_ptr(name.as_ptr() as _) })
            .to_str()
            .unwrap()
            .into())
    }

    /**
    Dump the data of a tuning.

    This function returns the only pitch values of a tuning.
     */
    pub fn tuning_dump_pitch(&self, bank: Bank, prog: Prog) -> Result<[f64; 128]> {
        let mut pitch = MaybeUninit::<[f64; 128]>::uninit();

        Synth::zero_ok(unsafe {
            self.handle
                .tuning_dump(bank as _, prog as _, null_mut(), 0, pitch.as_mut_ptr() as _)
        })?;
        Ok(unsafe { pitch.assume_init() })
    }
}

/**
The iterator over tunings
 */
pub struct TuningIter<'a> {
    handle: *mut engine::synth::Synth,
    phantom: PhantomData<&'a ()>,
    init: bool,
    next: bool,
}

impl<'a> TuningIter<'a> {
    fn from_ptr(handle: *mut engine::synth::Synth) -> Self {
        Self {
            handle,
            phantom: PhantomData,
            init: true,
            next: true,
        }
    }
}

impl<'a> Iterator for TuningIter<'a> {
    type Item = (Bank, Prog);

    fn next(&mut self) -> Option<Self::Item> {
        if self.init {
            self.init = false;
            unsafe {
                self.handle.as_mut().unwrap().tuning_iteration_start();
            }
        }
        if self.next {
            let mut bank = MaybeUninit::uninit();
            let mut prog = MaybeUninit::uninit();
            self.next = 0
                != unsafe {
                    self.handle
                        .as_mut()
                        .unwrap()
                        .tuning_iteration_next(bank.as_mut_ptr(), prog.as_mut_ptr())
                };

            Some((unsafe { bank.assume_init() as _ }, unsafe {
                prog.assume_init() as _
            }))
        } else {
            None
        }
    }
}
