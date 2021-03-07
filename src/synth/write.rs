use crate::Synth;

/// The trait which implements samples data buffer interface
pub trait IsSamples {
    fn write_samples(self, synth: &mut Synth);
}

impl IsSamples for &mut [i16] {
    /// Write samples interleaved
    fn write_samples(self, synth: &mut Synth) {
        let len = self.len() / 2;
        unsafe {
            let ptr = self as *mut _;
            synth.write_i16(len, &mut *ptr, 0, 2, &mut *ptr, 1, 2)
        }
    }
}

impl IsSamples for (&mut [i16], &mut [i16]) {
    /// Write samples non-interleaved
    fn write_samples(self, synth: &mut Synth) {
        let len = self.0.len().min(self.1.len());
        synth.write_i16(len, self.0, 0, 1, self.1, 0, 1)
    }
}

impl IsSamples for &mut [f32] {
    /// Write samples interleaved
    fn write_samples(self, synth: &mut Synth) {
        let len = self.len() / 2;
        synth.write_cb(len, 2, |id, l, r| {
            self[id] = l;
            self[id + 1] = r;
        });
    }
}

impl IsSamples for (&mut [f32], &mut [f32]) {
    /// Write samples non-interleaved
    fn write_samples(self, synth: &mut Synth) {
        let len = self.0.len().min(self.1.len());
        synth.write_cb(len, 1, |id, l, r| {
            self.0[id] = l;
            self.1[id] = r;
        });
    }
}

impl IsSamples for &mut [f64] {
    /// Write samples interleaved
    fn write_samples(self, synth: &mut Synth) {
        let len = self.len() / 2;
        synth.write_cb(len, 2, |id, l, r| {
            self[id] = l as f64;
            self[id + 1] = r as f64;
        });
    }
}

/**
Synthesizer plugin
 */
impl Synth {
    /**
    Write sound samples to the sample data buffer
     */
    pub fn write<S: IsSamples>(&mut self, samples: S) {
        samples.write_samples(self)
    }

    pub fn write_cb<F: FnMut(usize, f32, f32)>(&mut self, len: usize, incr: usize, cb: F) {
        self.handle.write(len, incr, cb)
    }

    /**
    Write samples as 16-bit signed integers

    # Safety

    The `len` must corresponds to the lenghtes of buffers.
     */
    #[allow(clippy::missing_safety_doc)] // TODO: Remove after closing https://github.com/rust-lang/rust-clippy/issues/5593
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn write_i16(
        &mut self,
        len: usize,
        left_out: &mut [i16],
        loff: u32,
        lincr: u32,
        right_out: &mut [i16],
        roff: u32,
        rincr: u32,
    ) {
        self.handle.write_s16(
            len as _, left_out, loff as _, lincr as _, right_out, roff as _, rincr as _,
        )
    }

    /**
    Write samples as 32-bit floating-point numbers

    # Safety

    The `len` must corresponds to the lenghtes of buffers.
     */
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn write_f32(
        &mut self,
        len: usize,
        left_out: &mut [f32],
        loff: u32,
        lincr: u32,
        right_out: &mut [f32],
        roff: u32,
        rincr: u32,
    ) {
        self.handle.write_f32(
            len as _, left_out, loff as _, lincr as _, right_out, roff as _, rincr as _,
        )
    }

    /**
    Write samples as 64-bit floating-point numbers

    # Safety

    The `len` must corresponds to the lenghtes of buffers.
     */
    #[allow(clippy::missing_safety_doc)] // TODO: Remove after closing https://github.com/rust-lang/rust-clippy/issues/5593
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn write_f64(
        &mut self,
        len: usize,
        left_out: &mut [f64],
        loff: u32,
        lincr: u32,
        right_out: &mut [f64],
        roff: u32,
        rincr: u32,
    ) {
        self.handle.write_f64(
            len as _, left_out, loff as _, lincr as _, right_out, roff as _, rincr as _,
        )
    }
}
