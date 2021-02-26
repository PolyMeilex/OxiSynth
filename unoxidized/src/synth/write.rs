use crate::synth::Synth;
use crate::synth::FLUID_SYNTH_PLAYING;
use crate::synth::RAND_TABLE;

impl Synth {
    pub unsafe fn write_f32(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut l;
        let left_out: *mut f32 = lout as *mut f32;
        let right_out: *mut f32 = rout as *mut f32;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        l = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if l == 64 as i32 {
                self.one_block(0 as i32);
                l = 0 as i32
            }
            *left_out.offset(j as isize) = *left_in.offset(l as isize);
            *right_out.offset(k as isize) = *right_in.offset(l as isize);
            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
        return 0 as i32;
    }

    pub unsafe fn write_f64(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut l;
        let left_out: *mut f64 = lout as *mut f64;
        let right_out: *mut f64 = rout as *mut f64;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        l = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if l == 64 as i32 {
                self.one_block(0 as i32);
                l = 0 as i32
            }
            *left_out.offset(j as isize) = *left_in.offset(l as isize) as f64;
            *right_out.offset(k as isize) = *right_in.offset(l as isize) as f64;
            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
        return 0 as i32;
    }

    pub unsafe fn write_s16(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut cur;
        let left_out: *mut i16 = lout as *mut i16;
        let right_out: *mut i16 = rout as *mut i16;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        let mut left_sample;
        let mut right_sample;
        let mut di: i32 = self.dither_index;
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        cur = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if cur == 64 as i32 {
                self.one_block(0 as i32);
                cur = 0 as i32
            }
            left_sample = roundi(
                *left_in.offset(cur as isize) * 32766.0f32
                    + RAND_TABLE[0 as i32 as usize][di as usize],
            ) as f32;
            right_sample = roundi(
                *right_in.offset(cur as isize) * 32766.0f32
                    + RAND_TABLE[1 as i32 as usize][di as usize],
            ) as f32;
            di += 1;
            if di >= 48000 as i32 {
                di = 0 as i32
            }
            if left_sample > 32767.0f32 {
                left_sample = 32767.0f32
            }
            if left_sample < -32768.0f32 {
                left_sample = -32768.0f32
            }
            if right_sample > 32767.0f32 {
                right_sample = 32767.0f32
            }
            if right_sample < -32768.0f32 {
                right_sample = -32768.0f32
            }
            *left_out.offset(j as isize) = left_sample as i16;
            *right_out.offset(k as isize) = right_sample as i16;
            i += 1;
            cur += 1;
            j += lincr;
            k += rincr
        }
        self.cur = cur;
        self.dither_index = di;
        return 0 as i32;
    }
}

fn roundi(x: f32) -> i32 {
    if x >= 0.0f32 {
        (x + 0.5f32) as i32
    } else {
        (x - 0.5f32) as i32
    }
}
