use super::channel::Channel;
use super::conv::concave;
use super::conv::convex;
use super::voice::Voice;

pub type ModFlags = u32;
pub const FLUID_MOD_CC: ModFlags = 16;
pub const FLUID_MOD_GC: ModFlags = 0;

pub const FLUID_MOD_LINEAR: ModFlags = 0;
const FLUID_MOD_CONVEX: ModFlags = 8;
const FLUID_MOD_CONCAVE: ModFlags = 4;
const FLUID_MOD_BIPOLAR: ModFlags = 2;
pub const FLUID_MOD_SWITCH: ModFlags = 12;

pub const FLUID_MOD_UNIPOLAR: ModFlags = 0;

pub const FLUID_MOD_NEGATIVE: ModFlags = 1;
pub const FLUID_MOD_POSITIVE: ModFlags = 0;

pub type ModSrc = u32;
pub const FLUID_MOD_VELOCITY: ModSrc = 2;
pub type GenType = u32;

pub const GEN_FILTERFC: GenType = 8;

use crate::gen::GenParam;

use soundfont_rs::data::modulator::{Modulator as SFModulator, ModulatorTransform};

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

        // Index of source 1, seven-bit value, SF2.01 section 8.2, page 50
        let src1 = (mod_src.src & 0b1111111) as u8;
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

        // index of source 2, seven-bit value, SF2.01 section 8.2, p.50
        let src2 = (mod_src.amt_src & 0b1111111) as u8;

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
        if mod_src.transform != ModulatorTransform::Linear {
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
        /* 'special treatment' for default controller
         *
         *  Reference: SF2.01 section 8.4.2
         *
         * The GM default controller 'vel-to-filter cut off' is not clearly
         * defined: If implemented according to the specs, the filter
         * frequency jumps between vel=63 and vel=64.  To maintain
         * compatibility with existing sound fonts, the implementation is
         * 'hardcoded', it is impossible to implement using only one
         * modulator otherwise.
         *
         * I assume here, that the 'intention' of the paragraph is one
         * octave (1200 cents) filter frequency shift between vel=127 and
         * vel=64.  'amount' is (-2400), at least as long as the controller
         * is set to default.
         *
         * Further, the 'appearance' of the modulator (source enumerator,
         * destination enumerator, flags etc) is different from that
         * described in section 8.4.2, but it matches the definition used in
         * several SF2.1 sound fonts (where it is used only to turn it off).
         * */
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

        let mut range1: f32 = 127.0f32;
        /* get the initial value of the first source */
        let mut v1 = if self.src1 as i32 > 0 as i32 {
            let v1 = if self.flags1 as i32 & FLUID_MOD_CC as i32 != 0 {
                chan.get_cc(self.src1 as i32) as f32
            } else {
                /* source 1 is one of the direct controllers */
                match self.src1 {
                    // FLUID_MOD_NONE
                    0 => range1,
                    // FLUID_MOD_VELOCITY
                    2 => voice.vel as f32,
                    // FLUID_MOD_KEY
                    3 => voice.key as f32,
                    // FLUID_MOD_KEYPRESSURE
                    10 => chan.key_pressure[voice.key as usize] as f32,
                    // FLUID_MOD_CHANNELPRESSURE
                    13 => chan.channel_pressure as f32,
                    // FLUID_MOD_PITCHWHEEL
                    14 => {
                        range1 = 0x4000 as f32;
                        chan.pitch_bend as f32
                    }
                    // FLUID_MOD_PITCHWHEELSENS
                    16 => chan.pitch_wheel_sensitivity as f32,
                    _ => 0.0,
                }
            };

            /* transform the input value */
            let v1 = match self.flags1 as i32 & 0xf as i32 {
                /* linear, unipolar, positive */
                0 => v1 / range1,
                /* linear, unipolar, negative */
                1 => 1.0 - v1 / range1,
                /* linear, bipolar, positive */
                2 => -1.0 + 2.0 * v1 / range1,
                /* linear, bipolar, negative */
                3 => 1.0 - 2.0 * v1 / range1,
                /* concave, unipolar, positive */
                4 => concave(v1),
                /* concave, unipolar, negative */
                5 => concave(127.0 - v1),
                /* concave, bipolar, positive */
                6 => {
                    if v1 > 64.0 {
                        concave(2.0 * (v1 - 64.0))
                    } else {
                        -concave(2.0 * (64.0 - v1))
                    }
                }
                /* concave, bipolar, negative */
                7 => {
                    if v1 > 64.0 {
                        -concave(2.0 * (v1 - 64.0))
                    } else {
                        concave(2.0 * (64.0 - v1))
                    }
                }
                /* convex, unipolar, positive */
                8 => convex(v1),
                /* convex, unipolar, negative */
                9 => convex(127.0 - v1),
                /* convex, bipolar, positive */
                10 => {
                    if v1 > 64.0 {
                        convex(2.0 * (v1 - 64.0))
                    } else {
                        -convex(2.0 * (64.0 - v1))
                    }
                }
                /* convex, bipolar, negative */
                11 => {
                    if v1 > 64.0 {
                        -convex(2.0 * (v1 - 64.0))
                    } else {
                        convex(2.0 * (64.0 - v1))
                    }
                }
                /* switch, unipolar, positive */
                12 => {
                    if v1 >= 64.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                /* switch, unipolar, negative */
                13 => {
                    if v1 >= 64.0 {
                        0.0
                    } else {
                        1.0
                    }
                }
                /* switch, bipolar, positive */
                14 => {
                    if v1 >= 64.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                /* switch, bipolar, negative */
                15 => {
                    if v1 >= 64.0 {
                        -1.0
                    } else {
                        1.0
                    }
                }
                _ => v1,
            };

            v1
        } else {
            return 0.0;
        };

        /* no need to go further */
        if v1 == 0.0 {
            return 0.0;
        }

        let range2: f32 = 127.0f32;
        /* get the second input source */
        let v2 = if self.src2 as i32 > 0 as i32 {
            let v2 = if self.flags2 as i32 & FLUID_MOD_CC as i32 != 0 {
                chan.get_cc(self.src2 as i32) as f32
            } else {
                match self.src2 {
                    // FLUID_MOD_NONE
                    0 => range2,
                    // FLUID_MOD_VELOCITY
                    2 => voice.vel as f32,
                    // FLUID_MOD_KEY
                    3 => voice.key as f32,
                    // FLUID_MOD_KEYPRESSURE
                    10 => chan.key_pressure[voice.key as usize] as f32,
                    // FLUID_MOD_CHANNELPRESSURE
                    13 => chan.channel_pressure as f32,
                    // FLUID_MOD_PITCHWHEEL
                    14 => chan.pitch_bend as f32,
                    // FLUID_MOD_PITCHWHEELSENS
                    16 => chan.pitch_wheel_sensitivity as f32,
                    _ => {
                        // https://github.com/divideconcept/FluidLite/blob/fdd05bad03cdb24d1f78b5fe3453842890c1b0e8/src/fluid_mod.c#L282
                        // why is this setting v1 to 0.0?
                        v1 = 0.0;
                        1.0
                    }
                }
            };

            /* transform the second input value */
            let v2 = match self.flags2 as i32 & 0xf as i32 {
                /* linear, unipolar, positive */
                0 => v2 / range2,
                /* linear, unipolar, negative */
                1 => 1.0 - v2 / range2,
                /* linear, bipolar, positive */
                2 => -1.0 + 2.0 * v2 / range2,
                /* linear, bipolar, negative */
                3 => -1.0 + 2.0 * v2 / range2,
                /* concave, unipolar, positive */
                4 => concave(v2),
                /* concave, unipolar, negative */
                5 => concave(127.0 - v2),
                /* concave, bipolar, positive */
                6 => {
                    if v2 > 64.0 {
                        concave(2.0 * (v2 - 64.0))
                    } else {
                        -concave(2.0 * (64.0 - v2))
                    }
                }
                /* concave, bipolar, negative */
                7 => {
                    if v2 > 64.0 {
                        -concave(2.0 * (v2 - 64.0))
                    } else {
                        concave(2.0 * (64.0 - v2))
                    }
                }
                /* convex, unipolar, positive */
                8 => convex(v2),
                /* convex, unipolar, negative */
                9 => 1.0 - convex(v2),
                /* convex, bipolar, positive */
                10 => {
                    if v2 > 64.0 {
                        -convex(2.0 * (v2 - 64.0))
                    } else {
                        convex(2.0 * (64.0 - v2))
                    }
                }
                /* convex, bipolar, negative */
                11 => {
                    if v2 > 64.0 {
                        -convex(2.0 * (v2 - 64.0))
                    } else {
                        convex(2.0 * (64.0 - v2))
                    }
                }
                /* switch, unipolar, positive */
                12 => {
                    if v2 >= 64.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                /* switch, unipolar, negative */
                13 => {
                    if v2 >= 64.0 {
                        0.0
                    } else {
                        1.0
                    }
                }
                /* switch, bipolar, positive */
                14 => {
                    if v2 >= 64.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                /* switch, bipolar, negative */
                15 => {
                    if v2 >= 64.0 {
                        -1.0
                    } else {
                        1.0
                    }
                }
                _ => v2,
            };

            v2
        } else {
            1.0
        };

        return self.amount as f32 * v1 * v2;
    }

    pub fn test_identity(&self, mod2: &Mod) -> bool {
        if self.dest != mod2.dest {
            false
        } else if self.src1 != mod2.src1 {
            false
        } else if self.src2 != mod2.src2 {
            false
        } else if self.flags1 != mod2.flags1 {
            false
        } else if self.flags2 != mod2.flags2 {
            false
        } else {
            true
        }
    }
}
