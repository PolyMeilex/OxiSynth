use std::sync::LazyLock;

use crate::core::Core;

static RAND_TABLE: LazyLock<[[f32; 48000]; 2]> = LazyLock::new(|| {
    let mut rand: [[f32; 48000]; 2] = [[0.0; 48000]; 2];

    for c in rand.iter_mut() {
        let mut dp = 0.0;

        for i in c.iter_mut().take(48000 - 1) {
            let r: i32 = rand::random();
            let d = r as f32 / 2147483647.0 - 0.5;
            *i = d - dp;
            dp = d;
        }

        c[48000 - 1] = 0.0 - dp;
    }

    rand
});

/// Stored in [`Synth::i16_output`]
#[derive(Default)]
pub(crate) struct I16OutputState {
    dither_index: usize,
}

#[inline]
pub fn write_i16(
    synth: &mut Core,
    len: usize,
    loff: usize,
    lincr: usize,
    roff: usize,
    rincr: usize,
    mut cb: impl FnMut((usize, i16), (usize, i16)),
) {
    let mut di = synth.i16_output.dither_index;

    let mut cur = synth.cur;
    let mut i = 0;
    let mut j = loff;
    let mut k = roff;

    while i < len {
        // fill up the buffers as needed
        if cur == 64 {
            synth.one_block(false);
            cur = 0;
        }

        // Converts stereo floating point sample data to signed 16 bit data with
        // dithering.

        let mut left_sample = f32::round(synth.left_buf[0][cur] * 32766.0 + RAND_TABLE[0][di]);
        let mut right_sample = f32::round(synth.right_buf[0][cur] * 32766.0 + RAND_TABLE[1][di]);

        di += 1;
        if di >= 48000 {
            di = 0;
        }

        // digital clipping
        left_sample = left_sample.clamp(-32768.0, 32767.0);
        right_sample = right_sample.clamp(-32768.0, 32767.0);

        cb(
            (j, left_sample as i16),
            (k, right_sample as i16),
            //
        );

        i += 1;
        cur += 1;
        j += lincr;
        k += rincr
    }

    synth.cur = cur;

    // keep dither buffer continuous
    synth.i16_output.dither_index = di;
}
