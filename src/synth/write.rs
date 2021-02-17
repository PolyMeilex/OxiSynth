use crate::{Status, Synth};

/// The trait which implements samples data buffer interface
pub trait IsSamples {
    fn write_samples(self, synth: &mut Synth) -> Status;
}

impl IsSamples for &mut [i16] {
    /// Write samples interleaved
    fn write_samples(self, synth: &mut Synth) -> Status {
        let len = self.len() / 2;
        unsafe { synth.write_i16(len, self.as_mut_ptr(), 0, 2, self.as_mut_ptr(), 1, 2) }
    }
}

impl IsSamples for (&mut [i16], &mut [i16]) {
    /// Write samples non-interleaved
    fn write_samples(self, synth: &mut Synth) -> Status {
        let len = self.0.len().min(self.1.len());
        unsafe { synth.write_i16(len, self.0.as_mut_ptr(), 0, 1, self.1.as_mut_ptr(), 0, 1) }
    }
}

impl IsSamples for &mut [f32] {
    /// Write samples interleaved
    fn write_samples(self, synth: &mut Synth) -> Status {
        let len = self.len() / 2;
        unsafe { synth.write_f32(len, self.as_mut_ptr(), 0, 2, self.as_mut_ptr(), 1, 2) }
    }
}

impl IsSamples for (&mut [f32], &mut [f32]) {
    /// Write samples non-interleaved
    fn write_samples(self, synth: &mut Synth) -> Status {
        let len = self.0.len().min(self.1.len());
        unsafe { synth.write_f32(len, self.0.as_mut_ptr(), 0, 1, self.1.as_mut_ptr(), 0, 1) }
    }
}

impl IsSamples for &mut [f64] {
    /// Write samples interleaved
    fn write_samples(self, synth: &mut Synth) -> Status {
        let len = self.len() / 2;
        unsafe { synth.write_f64(len, self.as_mut_ptr(), 0, 2, self.as_mut_ptr(), 1, 2) }
    }
}

/**
Synthesizer plugin
 */
impl Synth {
    /**
    Write sound samples to the sample data buffer
     */
    pub fn write<S: IsSamples>(&mut self, samples: S) -> Status {
        samples.write_samples(self)
    }

    /**
    Write samples as 16-bit signed integers

    # Safety

    The `len` must corresponds to the lenghtes of buffers.
     */
    #[allow(clippy::missing_safety_doc)] // TODO: Remove after closing https://github.com/rust-lang/rust-clippy/issues/5593
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub unsafe fn write_i16(
        &mut self,
        len: usize,
        lbuf: *mut i16,
        loff: u32,
        lincr: u32,
        rbuf: *mut i16,
        roff: u32,
        rincr: u32,
    ) -> Status {
        Synth::zero_ok(self.handle.write_s16(
            len as _, lbuf as _, loff as _, lincr as _, rbuf as _, roff as _, rincr as _,
        ))
    }

    /**
    Write samples as 32-bit floating-point numbers

    # Safety

    The `len` must corresponds to the lenghtes of buffers.
     */
    #[allow(clippy::missing_safety_doc)] // TODO: Remove after closing https://github.com/rust-lang/rust-clippy/issues/5593
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub unsafe fn write_f32(
        &mut self,
        len: usize,
        lbuf: *mut f32,
        loff: u32,
        lincr: u32,
        rbuf: *mut f32,
        roff: u32,
        rincr: u32,
    ) -> Status {
        Synth::zero_ok(self.handle.write_f32(
            len as _, lbuf as _, loff as _, lincr as _, rbuf as _, roff as _, rincr as _,
        ))
    }

    /**
    Write samples as 64-bit floating-point numbers

    # Safety

    The `len` must corresponds to the lenghtes of buffers.
     */
    #[allow(clippy::missing_safety_doc)] // TODO: Remove after closing https://github.com/rust-lang/rust-clippy/issues/5593
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub unsafe fn write_f64(
        &mut self,
        len: usize,
        lbuf: *mut f64,
        loff: u32,
        lincr: u32,
        rbuf: *mut f64,
        roff: u32,
        rincr: u32,
    ) -> Status {
        Synth::zero_ok(self.handle.write_f64(
            len as _, lbuf as _, loff as _, lincr as _, rbuf as _, roff as _, rincr as _,
        ))
    }
}
