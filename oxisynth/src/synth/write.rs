use crate::Synth;

/// The trait which implements samples data buffer interface
pub trait IsSamples {
    fn write_samples(self, synth: &mut Synth);
}

#[cfg(feature = "i16-out")]
impl IsSamples for &mut [i16] {
    /// Write samples interleaved
    #[inline(always)]
    fn write_samples(self, synth: &mut Synth) {
        let len = self.len() / 2;

        // interleaved
        synth.write_i16(len, 0, 2, 1, 2, |left, right| {
            self[left.0] = left.1;
            self[right.0] = right.1;
        });
    }
}

#[cfg(feature = "i16-out")]
impl IsSamples for (&mut [i16], &mut [i16]) {
    /// Write samples non-interleaved
    #[inline(always)]
    fn write_samples(self, synth: &mut Synth) {
        let len = self.0.len().min(self.1.len());

        synth.write_i16(len, 0, 1, 0, 1, |left, right| {
            self.0[left.0] = left.1;
            self.1[right.0] = right.1;
        });
    }
}

impl IsSamples for &mut [f32] {
    /// Write samples interleaved
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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

    pub fn read_next(&mut self) -> (f32, f32) {
        self.core.read_next()
    }

    #[inline(always)]
    pub fn write_cb<F: FnMut(usize, f32, f32)>(&mut self, len: usize, incr: usize, cb: F) {
        self.core.write(len, incr, cb)
    }

    /// Write samples as 16-bit signed integers
    ///
    /// ```ignore
    /// synth.write_i16(len, 0, 1, 0, 1, |left, right| {
    ///     left_buf[left.0] = left.1;
    ///     right_buf[right.0] = right.1;
    /// });
    /// ```
    #[cfg(feature = "i16-out")]
    #[inline(always)]
    pub fn write_i16(
        &mut self,
        len: usize,
        loff: usize,
        lincr: usize,
        roff: usize,
        rincr: usize,
        cb: impl FnMut((usize, i16), (usize, i16)),
    ) {
        self.core.write_i16(len, loff, lincr, roff, rincr, cb);
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
        self.core.write_f32(
            len as _, left_out, loff as _, lincr as _, right_out, roff as _, rincr as _,
        )
    }

    /**
    Write samples as 64-bit floating-point numbers

    # Safety

    The `len` must corresponds to the lenghtes of buffers.
     */
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
        self.core.write_f64(
            len as _, left_out, loff as _, lincr as _, right_out, roff as _, rincr as _,
        )
    }
}
