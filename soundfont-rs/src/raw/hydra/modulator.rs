use crate::error::Error;
use crate::raw::GeneratorType;

use super::super::utils::Reader;
use crate::riff::{Chunk, ChunkId, ScratchReader};
use std::io::{Read, Seek};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeneralPalette {
    /// 0: No controller is to be used. The output of this controller module should be treated as if its value were set to ‘1’. It should not be a means to turn off a modulator.
    NoController,
    /// 2: The controller source to be used is the velocity value which is sent from the MIDI note-on command which generated the given sound.
    NoteOnVelocity,
    /// 3: The controller source to be used is the key number value which was sent from the MIDI note-on command which generated the given sound.
    NoteOnKeyNumber,
    /// 10: The controller source to be used is the poly-pressure amount that is sent from the MIDI poly-pressure command.
    PolyPressure,
    /// 13: The controller source to be used is the channel pressure amount that is sent from the MIDI channel-pressure command.
    ChannelPressure,
    /// 14: The controller source to be used is the pitch wheel amount which is sent from the MIDI pitch wheel command.
    PitchWheel,
    /// 16: The controller source to be used is the pitch wheel sensitivity amount which is sent from the MIDI RPN 0 pitch wheel sensitivity command.
    PitchWheelSensitivity,
    /// 127: The controller source is the output of another modulator. This is NOT SUPPORTED as an Amount Source.
    Link,

    /// If such a value is encountered, the entire modulator structure should be ignored.
    Unknown(u8),
}

impl From<u8> for GeneralPalette {
    fn from(ty: u8) -> Self {
        match ty {
            0 => Self::NoController,
            2 => Self::NoteOnVelocity,
            3 => Self::NoteOnKeyNumber,
            10 => Self::PolyPressure,
            13 => Self::ChannelPressure,
            14 => Self::PitchWheel,
            16 => Self::PitchWheelSensitivity,
            127 => Self::Link,
            v => Self::Unknown(v),
        }
    }
}

// TODO: ControllerPalette should contain an index. probably like so...
// enum ControllerPalette {
//      General(GeneralEnum),
//      Midi(u8)
// }
//
/// 8.2.1 Source Enumerator Controller Palettes
///
/// The SoundFont format supports two distinct controller palettes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControllerPalette {
    /// General Controller palette of controllers is selected.
    ///
    /// The `index` field value corresponds to one of the following controller sources.
    /// - 0  No Controller
    /// - 2  Note-On Velocity
    /// - 3  Note-On Key Number
    /// - 10 Poly Pressure
    /// - 13 Channel Pressure
    /// - 14 Pitch Wheel
    /// - 16 Pitch Wheel Sensitivity
    /// - 127 Link
    General(GeneralPalette),
    /// MIDI Controller Palette is selected. The `index` field value corresponds to one of the 128 MIDI Continuous Controller messages as defined in the MIDI specification.
    Midi(u8),
}

/// 8.2.2 Source Directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceDirection {
    /// 0: The direction of the controller should be from the minimum value to the maximum value. So, for example, if the controller source is Key Number, then Key Number value of 0 corresponds to the minimum possible controller output, and Key Number value of 127 corresponds to the maximum possible controller input.
    Positive,
    /// 1: The direction of the controller should be from the maximum value to the minimum value. So, for example, if the controller source is Key Number, then a Key Number value of 0 corresponds to the maximum possible controller output, and the Key Number value of 127 corresponds to the minimum possible controller input.
    Negative,
}

// 8.2.3 Source Polarities
//
/// The SoundFont 2.01 format supports two polarities for any controller. The polarity if specified by bit 9 of the source enumeration field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourcePolarity {
    /// 0: The controller should be mapped with a minimum value of 0 and a maximum value of 1. This is also called Unipolar. Thus it behaves similar to the Modulation Wheel controller of the MIDI specification.
    Unipolar,
    /// 1: The controller sound be mapped with a minimum value of -1 and a maximum value of 1. This is also called Bipolar. Thus it behaves similar to the Pitch Wheel controller of the MIDI specification.
    Bipolar,
}

/// 8.2.4 Source Types
/// Specifies Continuity of the controller
///
/// The SoundFont 2.01 format may be used to support various types of controllers. This field completes the definition of the controller. A controller type specifies how the minimum value approaches the maximum value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceType {
    /// 0: The SoundFont modulator controller moves linearly from the minimum to the maximum value in the direction and with the polarity specified by the ‘D’ and ‘P’ bits.
    Linear,
    /// 1: The SoundFont modulator controller moves in a concave fashion from the minimum to the maximum value in the direction and with the polarity specified by the ‘D’ and ‘P’ bits. The concave characteristic follows variations of the mathematical equation:
    ///
    /// `output = log(sqrt(value^2)/(max value)^2)`
    Concave,
    /// 2: The SoundFont modulator controller moves in a convex fashion from the minimum to the maximum value in the direction and with the polarity specified by the ‘D’ and ‘P’ bits. The convex curve is the same curve as the concave curve, except the start and end points are reversed.
    Convex,
    /// 3: The SoundFont modulator controller output is at a minimum value while the controller input moves from the minimum to half of the maximum, after which the controller output is at a maximum. This occurs in the direction and with the polarity specified by the ‘D’ and ‘P’ bits.
    Switch,

    /// If such a value is encountered, the entire modulator structure should be ignored.
    Unknown(u8),
}

impl From<u8> for SourceType {
    fn from(ty: u8) -> Self {
        match ty {
            0 => Self::Linear,
            1 => Self::Concave,
            2 => Self::Convex,
            3 => Self::Switch,
            v => Self::Unknown(v),
        }
    }
}

/// 8.2  Modulator Source Enumerators
/// Flags telling the polarity of a modulator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModulatorSource {
    pub index: u8,
    pub controller_palette: ControllerPalette,
    pub direction: SourceDirection,
    pub polarity: SourcePolarity,
    /// Specifies Continuity of the controller
    pub ty: SourceType,
}

impl ModulatorSource {
    #[inline]
    pub fn is_linear(&self) -> bool {
        self.ty == SourceType::Linear
    }
    #[inline]
    pub fn is_concave(&self) -> bool {
        self.ty == SourceType::Concave
    }
    #[inline]
    pub fn is_convex(&self) -> bool {
        self.ty == SourceType::Convex
    }
    #[inline]
    pub fn is_switch(&self) -> bool {
        self.ty == SourceType::Switch
    }

    #[inline]
    pub fn is_unipolar(&self) -> bool {
        self.polarity == SourcePolarity::Unipolar
    }
    #[inline]
    pub fn is_bipolar(&self) -> bool {
        self.polarity == SourcePolarity::Bipolar
    }

    #[inline]
    pub fn is_positive(&self) -> bool {
        self.direction == SourceDirection::Positive
    }
    #[inline]
    pub fn is_negative(&self) -> bool {
        self.direction == SourceDirection::Negative
    }

    #[inline]
    pub fn is_cc(&self) -> bool {
        std::mem::discriminant(&self.controller_palette)
            == std::mem::discriminant(&ControllerPalette::Midi(0))
    }
    #[inline]
    pub fn is_gc(&self) -> bool {
        std::mem::discriminant(&self.controller_palette)
            == std::mem::discriminant(&ControllerPalette::General(GeneralPalette::NoController))
    }
}

impl From<u16> for ModulatorSource {
    fn from(src: u16) -> Self {
        // Index of source 1, seven-bit value, SF2.01 section 8.2, page 50
        let index: u8 = (src & 0b1111111)
            .try_into()
            .expect("Index is longer than 7 bits!");
        // Bit 7: CC flag SF 2.01 section 8.2.1 page 50
        let controller_palette = if src & 1 << 7 != 0 {
            ControllerPalette::Midi(index)
        } else {
            ControllerPalette::General(index.into())
        };

        // Bit 8: D flag SF 2.01 section 8.2.2 page 51
        let direction = if src & 1 << 8 != 0 {
            SourceDirection::Negative
        } else {
            SourceDirection::Positive
        };

        // Bit 9: P flag SF 2.01 section 8.2.3 page 51
        let polarity = if src & 1 << 9 != 0 {
            SourcePolarity::Bipolar
        } else {
            SourcePolarity::Unipolar
        };

        // modulator source types: SF2.01 section 8.2.1 page 52
        let ty = src >> 10;
        // type is a 6-bit value
        let ty: u8 = (ty & 0b111111)
            .try_into()
            .expect("Mod source type is longer than 6 bits!");
        let ty: SourceType = ty.into();

        Self {
            index,
            controller_palette,
            direction,
            polarity,
            ty,
        }
    }
}

#[allow(dead_code)]
/// 8.3  Modulator Transform Enumerators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModulatorTransform {
    /// 0: The output value of the multiplier is to be fed directly to the summing node of the given destination.
    Linear,
    /// 2: The output value of the multiplier is to be the absolute value of the input value, as defined by the relationship:
    ///
    /// `output = square root ((input value)^2)`
    ///
    /// or alternatively:
    ///
    /// `output = output * sgn(output)`
    Absolute,
}

impl TryFrom<u16> for ModulatorTransform {
    type Error = Error;
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Linear),
            2 => Ok(Self::Absolute),
            v => Err(Error::UnknownModulatorTransform(v)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Modulator {
    pub src: ModulatorSource,
    pub dest: GeneratorType,
    pub amount: i16,
    pub amt_src: ModulatorSource,
    pub transform: ModulatorTransform,
}

impl Modulator {
    pub(crate) fn read(reader: &mut Reader, terminal: bool) -> Result<Self, Error> {
        let mut src: u16 = reader.read_u16()?;
        let mut dest: u16 = reader.read_u16()?;
        let mut amount: i16 = reader.read_i16()?;
        let mut amt_src: u16 = reader.read_u16()?;
        let mut transform: u16 = reader.read_u16()?;

        // "The terminal record conventionally contains zero in all fields, and is always ignored."
        // This sadly is not always true, so let's zero it out ourselves.
        if terminal {
            src = 0;
            dest = 0;
            amount = 0;
            amt_src = 0;
            transform = 0;
        }

        Ok(Self {
            src: src.into(),
            dest: dest.try_into()?,
            amount,
            amt_src: amt_src.into(),
            transform: transform.try_into()?,
        })
    }

    pub(crate) fn read_all(
        pmod: &Chunk,
        file: &mut ScratchReader<impl Read + Seek>,
    ) -> Result<Vec<Self>, Error> {
        assert!(pmod.id() == ChunkId::pmod || pmod.id() == ChunkId::imod);

        let size = pmod.len();
        if size % 10 != 0 || size == 0 {
            Err(Error::InvalidModulatorChunkSize(size))
        } else {
            let amount = size / 10;

            let data = pmod.read_contents(file)?;
            let mut reader = Reader::new(data);

            (0..amount)
                .map(|id| Self::read(&mut reader, id == amount - 1))
                .collect()
        }
    }
}

/// 8.4  Default Modulators
pub mod default_modulators {
    use super::*;
    use crate::raw::GeneratorType;
    use SourceDirection::*;
    use SourcePolarity::*;
    use SourceType::*;

    const NO_CONTROLLER_SRC: ModulatorSource = ModulatorSource {
        index: 0,
        controller_palette: ControllerPalette::General(GeneralPalette::NoController),
        direction: Positive,
        polarity: Unipolar,
        ty: Linear,
    };

    /// 8.4.1  MIDI Note-On Velocity to Initial Attenuation
    pub static DEFAULT_VEL2ATT_MOD: Modulator = Modulator {
        dest: GeneratorType::InitialAttenuation,
        amount: 960,

        src: ModulatorSource {
            index: 2,
            controller_palette: ControllerPalette::General(GeneralPalette::NoteOnVelocity),
            direction: Negative,
            polarity: Unipolar,
            ty: Concave,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.2  MIDI Note-On Velocity to Filter Cutoff
    pub static DEFAULT_VEL2FILTER_MOD: Modulator = Modulator {
        dest: GeneratorType::InitialFilterFc,
        amount: -2400,

        src: ModulatorSource {
            index: 2,
            controller_palette: ControllerPalette::General(GeneralPalette::NoteOnVelocity),
            direction: Negative,
            polarity: Unipolar,
            ty: Linear,
        },

        // Note:
        // In SF2.01 it used to be 0x502
        // But in SF2.04 it is just 0x0
        // I believe that 0x502 was causing a problem in FS:
        // You can read about 0x502 problem here: https://github.com/FluidSynth/fluidsynth/blob/e4241469d49551b92478afbd2209939ff89441d5/src/synth/fluid_synth.c#L324
        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.3  MIDI Channel Pressure to Vibrato LFO Pitch Depth
    pub static DEFAULT_AT2VIBLFO_MOD: Modulator = Modulator {
        dest: GeneratorType::VibLfoToPitch,
        amount: 50,

        src: ModulatorSource {
            index: 13,
            controller_palette: ControllerPalette::General(GeneralPalette::ChannelPressure),
            direction: Positive,
            polarity: Unipolar,
            ty: Linear,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.4  MIDI Continuous Controller 1 to Vibrato LFO Pitch Depth
    pub static DEFAULT_MOD2VIBLFO_MOD: Modulator = Modulator {
        dest: GeneratorType::VibLfoToPitch,
        amount: 50,

        src: ModulatorSource {
            // Modulation Wheel or Lever
            index: 1,
            controller_palette: ControllerPalette::Midi(1),
            direction: Positive,
            polarity: Unipolar,
            ty: Linear,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.5  MIDI Continuous Controller 7 to Initial Attenuation
    pub static DEFAULT_ATT_MOD: Modulator = Modulator {
        dest: GeneratorType::InitialAttenuation,
        amount: 960,

        src: ModulatorSource {
            // Channel Volume (formerly Main Volume)
            index: 7,
            controller_palette: ControllerPalette::Midi(7),
            direction: Negative,
            polarity: Unipolar,
            ty: Concave,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.6  MIDI Continuous Controller 10 to Pan Position
    pub static DEFAULT_PAN_MOD: Modulator = Modulator {
        dest: GeneratorType::Pan,

        // Amount: 500. The SF specs 8.4.6, says: "Amount = 1000 tenths of a percent".
        // The center value (64) corresponds to 50%, so it follows that amount = 50% x 1000/% = 500.
        amount: 500,

        src: ModulatorSource {
            // Pan
            index: 10,
            controller_palette: ControllerPalette::Midi(10),
            direction: Positive,
            polarity: Bipolar,
            ty: Linear,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.7  MIDI Continuous Controller 11 to Initial Attenuation
    pub static DEFAULT_EXPR_MOD: Modulator = Modulator {
        dest: GeneratorType::InitialAttenuation,
        amount: 960,

        src: ModulatorSource {
            // Expression Controller
            index: 11,
            controller_palette: ControllerPalette::Midi(11),
            direction: Negative,
            polarity: Unipolar,
            ty: Concave,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.8  MIDI Continuous Controller 91 to Reverb Effects Send
    pub static DEFAULT_REVERB_MOD: Modulator = Modulator {
        dest: GeneratorType::ReverbEffectsSend,
        amount: 200,

        src: ModulatorSource {
            // Effects 1 Depth
            index: 91,
            controller_palette: ControllerPalette::Midi(91),
            direction: Positive,
            polarity: Unipolar,
            ty: Linear,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.9  MIDI Continuous Controller 93 to Chorus Effects Send
    pub static DEFAULT_CHORUS_MOD: Modulator = Modulator {
        dest: GeneratorType::ChorusEffectsSend,
        amount: 200,

        src: ModulatorSource {
            // Effects 3 Depth
            index: 93,
            controller_palette: ControllerPalette::Midi(93),
            direction: Positive,
            polarity: Unipolar,
            ty: Linear,
        },

        amt_src: NO_CONTROLLER_SRC,
        transform: ModulatorTransform::Linear,
    };

    /// 8.4.10  MIDI Pitch Wheel to Initial Pitch Controlled by MIDI Pitch Wheel Sensitivity
    ///
    /// Initial Pitch is not a "standard" generator (SF 2.04)
    ///
    /// That's why this mod is an const fn and
    /// user has to decide the destination themself.
    pub const fn default_pitch_bend_mod(dest: GeneratorType) -> Modulator {
        Modulator {
            dest,
            amount: 12700,

            src: ModulatorSource {
                index: 14,
                controller_palette: ControllerPalette::General(GeneralPalette::PitchWheel),
                direction: Positive,
                polarity: Bipolar,
                ty: Linear,
            },

            amt_src: ModulatorSource {
                index: 16,
                controller_palette: ControllerPalette::General(
                    GeneralPalette::PitchWheelSensitivity,
                ),
                direction: Positive,
                polarity: Unipolar,
                ty: Linear,
            },
            transform: ModulatorTransform::Linear,
        }
    }
}
