use super::super::channel_pool::Channel;
use super::super::conv::{concave, convex};
use super::super::voice_pool::Voice;

use super::generator::GeneratorType;

use soundfont::raw::{
    ControllerPalette, GeneralPalette, Modulator as SFModulator, ModulatorSource,
    ModulatorTransform, SourceDirection, SourcePolarity, SourceType,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mod {
    pub dest: GeneratorType,
    pub amount: f64,

    pub src: ModulatorSource,
    pub src2: ModulatorSource,
}

impl Mod {
    pub const fn const_from(mod_src: &SFModulator) -> Self {
        let mut amount = mod_src.amount as f64;

        let dest = mod_src.dest; // index of controlled generator

        if let SourceType::Unknown(_) = mod_src.src.ty {
            /* This shouldn't happen - unknown type!
             * Deactivate the modulator by setting the amount to 0. */
            amount = 0.0;
        }

        if let SourceType::Unknown(_) = mod_src.amt_src.ty {
            /* This shouldn't happen - unknown type!
             * Deactivate the modulator by setting the amount to 0. */
            amount = 0.0;
        }

        /* SF2.01 only uses the 'linear' transform (0).
         * Deactivate the modulator by setting the amount to 0 in any other case.
         */
        if mod_src.transform as u8 != ModulatorTransform::Linear as u8 {
            amount = 0.0;
        }

        Self {
            src: mod_src.src,
            amount,
            dest: GeneratorType::const_try_from(dest as u8).unwrap(), // index of controlled generator
            src2: mod_src.amt_src,
        }
    }
}

impl From<&SFModulator> for Mod {
    fn from(mod_src: &SFModulator) -> Self {
        Self::const_from(mod_src)
    }
}

impl Default for Mod {
    fn default() -> Self {
        Self {
            dest: GeneratorType::StartAddrOfs,
            src: 0.into(),
            src2: 0.into(),
            amount: 0.0,
        }
    }
}

impl Mod {
    pub fn get_dest(&self) -> GeneratorType {
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
        if self.src.controller_palette == ControllerPalette::General(GeneralPalette::NoteOnVelocity)
            && self.src.is_unipolar()
            && self.src.is_negative()
            && self.src.is_linear()
            && self.src2.controller_palette
                == ControllerPalette::General(GeneralPalette::NoteOnVelocity)
            && self.src2.is_unipolar()
            && self.src2.is_positive()
            && self.src2.is_switch()
            && self.dest == GeneratorType::FilterFc
        {
            return 0.0;
        }

        let mut range1: f32 = 127.0f32;
        /* get the initial value of the first source */
        let mut v1 = if self.src.index > 0 {
            use GeneralPalette::*;
            let v1 = match self.src.controller_palette {
                ControllerPalette::Midi(id) => chan.cc(id as usize) as f32,
                ControllerPalette::General(g) => match g {
                    NoController => range1,
                    NoteOnVelocity => voice.vel() as f32,
                    NoteOnKeyNumber => voice.key() as f32,
                    PolyPressure => chan.key_pressure(voice.key() as usize) as f32,
                    ChannelPressure => chan.channel_pressure() as f32,
                    PitchWheel => {
                        range1 = 0x4000 as f32;
                        chan.pitch_bend() as f32
                    }
                    PitchWheelSensitivity => chan.pitch_wheel_sensitivity() as f32,
                    _ => 0.0,
                },
            };

            use SourceDirection::*;
            use SourcePolarity::*;
            use SourceType::*;

            match (self.src.ty, self.src.polarity, self.src.direction) {
                // 0
                (Linear, Unipolar, Positive) => v1 / range1,
                // 1
                (Linear, Unipolar, Negative) => 1.0 - v1 / range1,
                // 2
                (Linear, Bipolar, Positive) => -1.0 + 2.0 * v1 / range1,
                // 3
                (Linear, Bipolar, Negative) => 1.0 - 2.0 * v1 / range1,

                // 4
                (Concave, Unipolar, Positive) => concave(v1),
                // 5
                (Concave, Unipolar, Negative) => concave(127.0 - v1),
                // 6
                (Concave, Bipolar, Positive) => {
                    if v1 > 64.0 {
                        concave(2.0 * (v1 - 64.0))
                    } else {
                        -concave(2.0 * (64.0 - v1))
                    }
                }
                // 7
                (Concave, Bipolar, Negative) => {
                    if v1 > 64.0 {
                        -concave(2.0 * (v1 - 64.0))
                    } else {
                        concave(2.0 * (64.0 - v1))
                    }
                }

                // 8
                (Convex, Unipolar, Positive) => convex(v1),
                // 9
                (Convex, Unipolar, Negative) => convex(127.0 - v1),
                // 10
                (Convex, Bipolar, Positive) => {
                    if v1 > 64.0 {
                        convex(2.0 * (v1 - 64.0))
                    } else {
                        -convex(2.0 * (64.0 - v1))
                    }
                }
                // 11
                (Convex, Bipolar, Negative) => {
                    if v1 > 64.0 {
                        -convex(2.0 * (v1 - 64.0))
                    } else {
                        convex(2.0 * (64.0 - v1))
                    }
                }

                // 12
                (Switch, Unipolar, Positive) => {
                    if v1 >= 64.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                // 13
                (Switch, Unipolar, Negative) => {
                    if v1 >= 64.0 {
                        0.0
                    } else {
                        1.0
                    }
                }
                // 14
                (Switch, Bipolar, Positive) => {
                    if v1 >= 64.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                // 15
                (Switch, Bipolar, Negative) => {
                    if v1 >= 64.0 {
                        -1.0
                    } else {
                        1.0
                    }
                }

                _ => v1,
            }
        } else {
            return 0.0;
        };

        /* no need to go further */
        if v1 == 0.0 {
            return 0.0;
        }

        let range2: f32 = 127.0f32;
        /* get the second input source */
        let v2 = if self.src2.index > 0 {
            use GeneralPalette::*;
            let v2 = match self.src2.controller_palette {
                ControllerPalette::Midi(id) => chan.cc(id as usize) as f32,
                ControllerPalette::General(g) => match g {
                    NoController => range2,
                    NoteOnVelocity => voice.vel() as f32,
                    NoteOnKeyNumber => voice.key() as f32,
                    PolyPressure => chan.key_pressure(voice.key() as usize) as f32,
                    ChannelPressure => chan.channel_pressure() as f32,
                    PitchWheel => chan.pitch_bend() as f32,
                    PitchWheelSensitivity => chan.pitch_wheel_sensitivity() as f32,
                    _ => {
                        // https://github.com/divideconcept/FluidLite/blob/fdd05bad03cdb24d1f78b5fe3453842890c1b0e8/src/fluid_mod.c#L282
                        // why is this setting v1 to 0.0?
                        v1 = 0.0;
                        1.0
                    }
                },
            };

            use SourceDirection::*;
            use SourcePolarity::*;
            use SourceType::*;

            /* transform the second input value */

            match (self.src2.ty, self.src2.polarity, self.src2.direction) {
                // 0
                (Linear, Unipolar, Positive) => v2 / range2,
                // 1
                (Linear, Unipolar, Negative) => 1.0 - v2 / range2,
                // 2
                (Linear, Bipolar, Positive) => -1.0 + 2.0 * v2 / range2,
                // 3
                (Linear, Bipolar, Negative) => -1.0 + 2.0 * v2 / range2,

                // 4
                (Concave, Unipolar, Positive) => concave(v2),
                // 5
                (Concave, Unipolar, Negative) => concave(127.0 - v2),
                // 6
                (Concave, Bipolar, Positive) => {
                    if v2 > 64.0 {
                        concave(2.0 * (v2 - 64.0))
                    } else {
                        -concave(2.0 * (64.0 - v2))
                    }
                }
                // 7
                (Concave, Bipolar, Negative) => {
                    if v2 > 64.0 {
                        -concave(2.0 * (v2 - 64.0))
                    } else {
                        concave(2.0 * (64.0 - v2))
                    }
                }

                // 8
                (Convex, Unipolar, Positive) => convex(v2),
                // 9
                (Convex, Unipolar, Negative) => 1.0 - convex(v2),
                // 10
                (Convex, Bipolar, Positive) => {
                    if v2 > 64.0 {
                        -convex(2.0 * (v2 - 64.0))
                    } else {
                        convex(2.0 * (64.0 - v2))
                    }
                }
                // 11
                (Convex, Bipolar, Negative) => {
                    if v2 > 64.0 {
                        -convex(2.0 * (v2 - 64.0))
                    } else {
                        convex(2.0 * (64.0 - v2))
                    }
                }

                // 12
                (Switch, Unipolar, Positive) => {
                    if v2 >= 64.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                // 13
                (Switch, Unipolar, Negative) => {
                    if v2 >= 64.0 {
                        0.0
                    } else {
                        1.0
                    }
                }
                // 14
                (Switch, Bipolar, Positive) => {
                    if v2 >= 64.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                // 15
                (Switch, Bipolar, Negative) => {
                    if v2 >= 64.0 {
                        -1.0
                    } else {
                        1.0
                    }
                }

                _ => v2,
            }
        } else {
            1.0
        };

        self.amount as f32 * v1 * v2
    }

    pub fn test_identity(&self, mod2: &Mod) -> bool {
        if self.dest != mod2.dest {
            return false;
        }

        if self.src != mod2.src {
            return false;
        }

        self.src2 == mod2.src2
    }
}

pub mod default {
    use super::Mod;
    use soundfont::raw::{default_modulators, GeneratorType};

    /// 8.4.1  MIDI Note-On Velocity to Initial Attenuation
    pub const DEFAULT_VEL2ATT_MOD: Mod = Mod::const_from(&default_modulators::DEFAULT_VEL2ATT_MOD);

    /// 8.4.2  MIDI Note-On Velocity to Filter Cutoff
    pub const DEFAULT_VEL2FILTER_MOD: Mod =
        Mod::const_from(&default_modulators::DEFAULT_VEL2FILTER_MOD);

    /// 8.4.3  MIDI Channel Pressure to Vibrato LFO Pitch Depth
    pub const DEFAULT_AT2VIBLFO_MOD: Mod =
        Mod::const_from(&default_modulators::DEFAULT_AT2VIBLFO_MOD);

    /// 8.4.4  MIDI Continuous Controller 1 to Vibrato LFO Pitch Depth
    pub const DEFAULT_MOD2VIBLFO_MOD: Mod =
        Mod::const_from(&default_modulators::DEFAULT_MOD2VIBLFO_MOD);

    /// 8.4.5  MIDI Continuous Controller 7 to Initial Attenuation
    pub const DEFAULT_ATT_MOD: Mod = Mod::const_from(&default_modulators::DEFAULT_ATT_MOD);

    /// 8.4.6  MIDI Continuous Controller 10 to Pan Position
    pub const DEFAULT_PAN_MOD: Mod = Mod::const_from(&default_modulators::DEFAULT_PAN_MOD);

    /// 8.4.7  MIDI Continuous Controller 11 to Initial Attenuation
    pub const DEFAULT_EXPR_MOD: Mod = Mod::const_from(&default_modulators::DEFAULT_EXPR_MOD);

    /// 8.4.8  MIDI Continuous Controller 91 to Reverb Effects Send
    pub const DEFAULT_REVERB_MOD: Mod = Mod::const_from(&default_modulators::DEFAULT_REVERB_MOD);

    /// 8.4.9  MIDI Continuous Controller 93 to Chorus Effects Send
    pub const DEFAULT_CHORUS_MOD: Mod = Mod::const_from(&default_modulators::DEFAULT_CHORUS_MOD);

    /// 8.4.10  MIDI Pitch Wheel to Initial Pitch Controlled by MIDI Pitch Wheel Sensitivity
    ///
    /// GeneratorType::Unused5 (59) corresponds to gen::GenParam::Pitch (59)
    pub const DEFAULT_PITCH_BEND_MOD: Mod = Mod::const_from(
        &default_modulators::default_pitch_bend_mod(GeneratorType::Unused5),
    );
}
