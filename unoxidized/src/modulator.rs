use super::channel::Channel;
use super::conv::fluid_concave;
use super::conv::fluid_convex;
use super::voice::Voice;
pub type ModFlags = u32;
pub const FLUID_MOD_CC: ModFlags = 16;
pub const FLUID_MOD_GC: ModFlags = 0;
pub const FLUID_MOD_SWITCH: ModFlags = 12;
pub const FLUID_MOD_LINEAR: ModFlags = 0;
pub const FLUID_MOD_UNIPOLAR: ModFlags = 0;
pub const FLUID_MOD_NEGATIVE: ModFlags = 1;
pub const FLUID_MOD_POSITIVE: ModFlags = 0;
pub type ModSrc = u32;
pub const FLUID_MOD_VELOCITY: ModSrc = 2;
pub type GenType = u32;
pub const GEN_FILTERFC: GenType = 8;

pub struct Mod {
    pub(crate) dest: u8,
    pub(crate) src1: u8,
    pub(crate) flags1: u8,
    pub(crate) src2: u8,
    pub(crate) flags2: u8,
    pub(crate) amount: f64,
    pub(crate) next: *mut Mod,
}

impl Clone for Mod {
    fn clone(&self) -> Self {
        Self {
            dest: self.dest,
            src1: self.src1,
            flags1: self.flags1,
            src2: self.src2,
            flags2: self.flags2,
            amount: self.amount,
            next: 0 as _,
        }
    }
}

impl Mod {
    pub fn new() -> *mut Mod {
        return Box::into_raw(Box::new(Mod {
            dest: 0,
            src1: 0,
            flags1: 0,
            src2: 0,
            flags2: 0,
            amount: 0f64,
            next: 0 as _,
        }));
    }

    pub fn delete(&mut self) {
        unsafe {
            Box::from_raw(self);
        }
    }

    pub unsafe fn set_source1(&mut self, src: i32, flags: i32) {
        self.src1 = src as u8;
        self.flags1 = flags as u8;
    }

    pub unsafe fn set_source2(&mut self, src: i32, flags: i32) {
        self.src2 = src as u8;
        self.flags2 = flags as u8;
    }

    pub unsafe fn set_dest(&mut self, dest: i32) {
        self.dest = dest as u8;
    }

    pub unsafe fn set_amount(&mut self, amount: f64) {
        self.amount = amount;
    }

    pub fn get_dest(&self) -> i32 {
        return self.dest as i32;
    }

    pub fn get_value(&mut self, chan: &mut Channel, voice: &mut Voice) -> f32 {
        let mut v1: f32;
        let mut v2: f32 = 1.0f32;
        let mut range1: f32 = 127.0f32;
        let range2: f32 = 127.0f32;
        if self.src2 as i32 == FLUID_MOD_VELOCITY as i32
            && self.src1 as i32 == FLUID_MOD_VELOCITY as i32
            && self.flags1 as i32
                == FLUID_MOD_GC as i32
                    | FLUID_MOD_UNIPOLAR as i32
                    | FLUID_MOD_NEGATIVE as i32
                    | FLUID_MOD_LINEAR as i32
            && self.flags2 as i32
                == FLUID_MOD_GC as i32
                    | FLUID_MOD_UNIPOLAR as i32
                    | FLUID_MOD_POSITIVE as i32
                    | FLUID_MOD_SWITCH as i32
            && self.dest as i32 == GEN_FILTERFC as i32
        {
            return 0 as i32 as f32;
        }
        if self.src1 as i32 > 0 as i32 {
            if self.flags1 as i32 & FLUID_MOD_CC as i32 != 0 {
                v1 = chan.get_cc(self.src1 as i32) as f32
            } else {
                match self.src1 as i32 {
                    0 => v1 = range1,
                    2 => v1 = voice.vel as f32,
                    3 => v1 = voice.key as f32,
                    10 => v1 = chan.key_pressure[voice.key as usize] as f32,
                    13 => v1 = chan.channel_pressure as f32,
                    14 => {
                        v1 = chan.pitch_bend as f32;
                        range1 = 0x4000 as i32 as f32
                    }
                    16 => v1 = chan.pitch_wheel_sensitivity as f32,
                    _ => v1 = 0.0f32,
                }
            }
            match self.flags1 as i32 & 0xf as i32 {
                0 => v1 /= range1,
                1 => v1 = 1.0f32 - v1 / range1,
                2 => v1 = -1.0f32 + 2.0f32 * v1 / range1,
                3 => v1 = 1.0f32 - 2.0f32 * v1 / range1,
                4 => v1 = fluid_concave(v1),
                5 => v1 = fluid_concave(127 as i32 as f32 - v1),
                6 => {
                    v1 = if v1 > 64 as i32 as f32 {
                        fluid_concave(2 as i32 as f32 * (v1 - 64 as i32 as f32))
                    } else {
                        -fluid_concave(2 as i32 as f32 * (64 as i32 as f32 - v1))
                    }
                }
                7 => {
                    v1 = if v1 > 64 as i32 as f32 {
                        -fluid_concave(2 as i32 as f32 * (v1 - 64 as i32 as f32))
                    } else {
                        fluid_concave(2 as i32 as f32 * (64 as i32 as f32 - v1))
                    }
                }
                8 => v1 = fluid_convex(v1),
                9 => v1 = fluid_convex(127 as i32 as f32 - v1),
                10 => {
                    v1 = if v1 > 64 as i32 as f32 {
                        fluid_convex(2 as i32 as f32 * (v1 - 64 as i32 as f32))
                    } else {
                        -fluid_convex(2 as i32 as f32 * (64 as i32 as f32 - v1))
                    }
                }
                11 => {
                    v1 = if v1 > 64 as i32 as f32 {
                        -fluid_convex(2 as i32 as f32 * (v1 - 64 as i32 as f32))
                    } else {
                        fluid_convex(2 as i32 as f32 * (64 as i32 as f32 - v1))
                    }
                }
                12 => {
                    v1 = if v1 >= 64 as i32 as f32 {
                        1.0f32
                    } else {
                        0.0f32
                    }
                }
                13 => {
                    v1 = if v1 >= 64 as i32 as f32 {
                        0.0f32
                    } else {
                        1.0f32
                    }
                }
                14 => {
                    v1 = if v1 >= 64 as i32 as f32 {
                        1.0f32
                    } else {
                        -1.0f32
                    }
                }
                15 => {
                    v1 = if v1 >= 64 as i32 as f32 {
                        -1.0f32
                    } else {
                        1.0f32
                    }
                }
                _ => {}
            }
        } else {
            return 0.0f32;
        }
        if v1 == 0.0f32 {
            return 0.0f32;
        }
        if self.src2 as i32 > 0 as i32 {
            if self.flags2 as i32 & FLUID_MOD_CC as i32 != 0 {
                v2 = chan.get_cc(self.src2 as i32) as f32
            } else {
                match self.src2 as i32 {
                    0 => v2 = range2,
                    2 => v2 = voice.vel as f32,
                    3 => v2 = voice.key as f32,
                    10 => v2 = chan.key_pressure[voice.key as usize] as f32,
                    13 => v2 = chan.channel_pressure as f32,
                    14 => v2 = chan.pitch_bend as f32,
                    16 => v2 = chan.pitch_wheel_sensitivity as f32,
                    _ => v1 = 0.0f32,
                }
            }
            match self.flags2 as i32 & 0xf as i32 {
                0 => v2 /= range2,
                1 => v2 = 1.0f32 - v2 / range2,
                2 => v2 = -1.0f32 + 2.0f32 * v2 / range2,
                3 => v2 = -1.0f32 + 2.0f32 * v2 / range2,
                4 => v2 = fluid_concave(v2),
                5 => v2 = fluid_concave(127 as i32 as f32 - v2),
                6 => {
                    v2 = if v2 > 64 as i32 as f32 {
                        fluid_concave(2 as i32 as f32 * (v2 - 64 as i32 as f32))
                    } else {
                        -fluid_concave(2 as i32 as f32 * (64 as i32 as f32 - v2))
                    }
                }
                7 => {
                    v2 = if v2 > 64 as i32 as f32 {
                        -fluid_concave(2 as i32 as f32 * (v2 - 64 as i32 as f32))
                    } else {
                        fluid_concave(2 as i32 as f32 * (64 as i32 as f32 - v2))
                    }
                }
                8 => v2 = fluid_convex(v2),
                9 => v2 = 1.0f32 - fluid_convex(v2),
                10 => {
                    v2 = if v2 > 64 as i32 as f32 {
                        -fluid_convex(2 as i32 as f32 * (v2 - 64 as i32 as f32))
                    } else {
                        fluid_convex(2 as i32 as f32 * (64 as i32 as f32 - v2))
                    }
                }
                11 => {
                    v2 = if v2 > 64 as i32 as f32 {
                        -fluid_convex(2 as i32 as f32 * (v2 - 64 as i32 as f32))
                    } else {
                        fluid_convex(2 as i32 as f32 * (64 as i32 as f32 - v2))
                    }
                }
                12 => {
                    v2 = if v2 >= 64 as i32 as f32 {
                        1.0f32
                    } else {
                        0.0f32
                    }
                }
                13 => {
                    v2 = if v2 >= 64 as i32 as f32 {
                        0.0f32
                    } else {
                        1.0f32
                    }
                }
                14 => {
                    v2 = if v2 >= 64 as i32 as f32 {
                        1.0f32
                    } else {
                        -1.0f32
                    }
                }
                15 => {
                    v2 = if v2 >= 64 as i32 as f32 {
                        -1.0f32
                    } else {
                        1.0f32
                    }
                }
                _ => {}
            }
        } else {
            v2 = 1.0f32
        }
        return self.amount as f32 * v1 * v2;
    }

    pub fn test_identity(&self, mod2: &Mod) -> i32 {
        if self.dest as i32 != mod2.dest as i32 {
            return 0 as i32;
        }
        if self.src1 as i32 != mod2.src1 as i32 {
            return 0 as i32;
        }
        if self.src2 as i32 != mod2.src2 as i32 {
            return 0 as i32;
        }
        if self.flags1 as i32 != mod2.flags1 as i32 {
            return 0 as i32;
        }
        if self.flags2 as i32 != mod2.flags2 as i32 {
            return 0 as i32;
        }
        return 1 as i32;
    }
}
