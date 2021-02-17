use crate::{engine, Error, Result, Status, Synth};
use std::ffi::CStr;

impl Synth {
    /**
    Get a textual representation of the last error
     */
    pub(super) fn error() -> String {
        let error = unsafe { engine::synth::error() };
        let error = unsafe { CStr::from_ptr(error as _) };
        error.to_str().unwrap().into()
    }

    pub(super) fn neg_err(ret: i32) -> Result<i32> {
        if ret < 0 {
            Err(Error::Fluid(Synth::error()))
        } else {
            Ok(ret)
        }
    }

    pub(super) fn zero_ok(ret: i32) -> Status {
        if ret == 0 {
            Ok(())
        } else {
            Err(Error::Fluid(Synth::error()))
        }
    }
}
