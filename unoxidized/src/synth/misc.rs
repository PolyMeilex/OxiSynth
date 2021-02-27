use crate::synth::FLUID_ERRBUF;

pub unsafe fn error() -> *mut u8 {
    FLUID_ERRBUF.as_mut_ptr()
}
