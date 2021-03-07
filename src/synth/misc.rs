use crate::Synth;

impl Synth {
    pub(super) fn zero_ok(ret: i32) -> Result<(), ()> {
        if ret == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}
