use crate::{Error, Status, Synth};

impl Synth {
    pub(super) fn zero_ok(ret: i32) -> Status {
        if ret == 0 {
            Ok(())
        } else {
            Err(Error::Fluid("".into()))
        }
    }
}
