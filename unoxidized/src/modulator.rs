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

const FLUID_MOD_CONVEX: ModFlags = 8;
const FLUID_MOD_CONCAVE: ModFlags = 4;
const FLUID_MOD_BIPOLAR: ModFlags = 2;

use crate::gen::GenParam;

use soundfont_rs::data::modulator::SFModulator;

#[derive(Copy, Debug, PartialEq)]
pub(crate) struct Mod {
    pub(crate) dest: GenParam,
    pub(crate) amount: f64,

    pub(crate) src1: u8,
    pub(crate) flags1: u8,

    pub(crate) src2: u8,
    pub(crate) flags2: u8,
}

impl From<&SFModulator> for Mod {
    fn from(mod_src: &SFModulator) -> Self {
        // Amount
        let mut amount = mod_src.amount as f64;

        // Source
        let src1 = (mod_src.src & 127) as u8;
        let flags1 = {
            let mut flags1 = 0u8;

            // Bit 7: CC flag SF 2.01 section 8.2.1 page 50
            if mod_src.src & 1 << 7 != 0 {
                flags1 |= FLUID_MOD_CC as u8;
            } else {
                flags1 |= FLUID_MOD_GC as u8;
            }

            // Bit 8: D flag SF 2.01 section 8.2.2 page 51
            if mod_src.src & 1 << 8 != 0 {
                flags1 |= FLUID_MOD_NEGATIVE as u8;
            } else {
                flags1 |= FLUID_MOD_POSITIVE as u8;
            }

            // Bit 9: P flag SF 2.01 section 8.2.3 page 51
            if mod_src.src & 1 << 9 != 0 {
                flags1 |= FLUID_MOD_BIPOLAR as u8;
            } else {
                flags1 |= FLUID_MOD_UNIPOLAR as u8;
            }

            // modulator source types: SF2.01 section 8.2.1 page 52
            let mut type_0 = mod_src.src >> 10;
            type_0 &= 63; // type is a 6-bit value
            if type_0 == 0 {
                flags1 |= FLUID_MOD_LINEAR as u8;
            } else if type_0 == 1 {
                flags1 |= FLUID_MOD_CONCAVE as u8;
            } else if type_0 == 2 {
                flags1 |= FLUID_MOD_CONVEX as u8;
            } else if type_0 == 3 {
                flags1 |= FLUID_MOD_SWITCH as u8;
            } else {
                /* This shouldn't happen - unknown type!
                 * Deactivate the modulator by setting the amount to 0. */
                amount = 0.0;
            }
            flags1
        };

        // Dest
        let dest = mod_src.dest as u8; // index of controlled generator
        let src2 = (mod_src.amt_src & 127) as u8; // index of source 2, seven-bit value, SF2.01 section 8.2, p.50

        // Amount source
        let flags2 = {
            let mut flags2 = 0;
            // Bit 7: CC flag SF 2.01 section 8.2.1 page 50
            if mod_src.amt_src & 1 << 7 != 0 {
                flags2 |= FLUID_MOD_CC as u8;
            } else {
                flags2 |= FLUID_MOD_GC as u8;
            }

            // Bit 8: D flag SF 2.01 section 8.2.2 page 51
            if mod_src.amt_src & 1 << 8 != 0 {
                flags2 |= FLUID_MOD_NEGATIVE as u8;
            } else {
                flags2 |= FLUID_MOD_POSITIVE as u8;
            }

            // Bit 9: P flag SF 2.01 section 8.2.3 page 51
            if mod_src.amt_src as i32 & (1 as i32) << 9 as i32 != 0 {
                flags2 |= FLUID_MOD_BIPOLAR as u8;
            } else {
                flags2 |= FLUID_MOD_UNIPOLAR as u8;
            }

            // modulator source types: SF2.01 section 8.2.1 page 52
            let mut type_0 = mod_src.amt_src >> 10;
            type_0 &= 63; // type is a 6-bit value
            if type_0 == 0 {
                flags2 |= FLUID_MOD_LINEAR as u8;
            } else if type_0 == 1 {
                flags2 |= FLUID_MOD_CONCAVE as u8;
            } else if type_0 == 2 {
                flags2 |= FLUID_MOD_CONVEX as u8;
            } else if type_0 == 3 {
                flags2 |= FLUID_MOD_SWITCH as u8;
            } else {
                /* This shouldn't happen - unknown type!
                 * Deactivate the modulator by setting the amount to 0. */
                amount = 0.0;
            }
            flags2
        };

        // Transform

        /* SF2.01 only uses the 'linear' transform (0).
         * Deactivate the modulator by setting the amount to 0 in any other case.
         */
        if mod_src.transform as i32 != 0 as i32 {
            amount = 0.0;
        }

        use num_traits::FromPrimitive;

        Mod {
            src1,
            amount,
            flags1,
            dest: FromPrimitive::from_u8(dest).unwrap(), // index of controlled generator
            src2, // index of source 2, seven-bit value, SF2.01 section 8.2, p.50
            flags2,
        }
    }
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
        }
    }
}

impl Default for Mod {
    fn default() -> Self {
        Self {
            dest: GenParam::StartAddrOfs,
            src1: 0,
            flags1: 0,
            src2: 0,
            flags2: 0,
            amount: 0.0,
        }
    }
}

impl Mod {
    pub fn get_dest(&self) -> GenParam {
        self.dest
    }

    pub fn get_value(&self, chan: &Channel, voice: &Voice) -> f32 {
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

    pub fn test_identity(&self, mod2: &Mod) -> bool {
        if self.dest as i32 != mod2.dest as i32 {
            false
        } else if self.src1 as i32 != mod2.src1 as i32 {
            false
        } else if self.src2 as i32 != mod2.src2 as i32 {
            false
        } else if self.flags1 as i32 != mod2.flags1 as i32 {
            false
        } else if self.flags2 as i32 != mod2.flags2 as i32 {
            false
        } else {
            true
        }
    }
}
