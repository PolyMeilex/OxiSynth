use crate::error::ParseError;

use super::super::utils::Reader;
use riff::Chunk;
use std::io::{Read, Seek};

/// 8.2.1 Source Enumerator Controller Palettes
pub enum ControllerPalette {
    General,
    Midi,
}

/// 8.2.2 Source Directions
pub enum SourceDirection {
    Positive,
    Negative,
}

// 8.2.3 Source Polarities
pub enum SourcePolarity {
    Unipolar,
    Bipolar,
}

/// 8.2.4 Source Types
/// Specifies Continuity of the controller
pub enum SourceTypes {
    Linear,
    Concave,
    Convex,
    Switch,
}

#[allow(dead_code)]
/// 8.2  Modulator Source Enumerators  
/// Flags telling the polarity of a modulator.
pub struct ModulatorSource {
    index: u8,
    controller_palette: ControllerPalette,
    direction: SourceDirection,
    polarity: SourcePolarity,
    /// Specifies Continuity of the controller
    src_type: SourceTypes,
}

#[derive(Debug, Clone)]
pub struct Modulator {
    pub src: u16,  // TODO: ModulatorSource
    pub dest: u16, // TODO: SFGeneratorType
    pub amount: i16,
    pub amt_src: u16,
    pub transform: u16,
}

impl Modulator {
    pub fn read(reader: &mut Reader) -> Result<Self, ParseError> {
        let src: u16 = reader.read_u16()?;
        let dest: u16 = reader.read_u16()?;
        let amount: i16 = reader.read_i16()?;
        let amt_src: u16 = reader.read_u16()?;
        let transform: u16 = reader.read_u16()?;

        Ok(Self {
            src,
            dest,
            amount,
            amt_src,
            transform,
        })
    }

    pub fn read_all<F: Read + Seek>(pmod: &Chunk, file: &mut F) -> Result<Vec<Self>, ParseError> {
        assert!(pmod.id().as_str() == "pmod" || pmod.id().as_str() == "imod");

        let size = pmod.len();
        if size % 10 != 0 || size == 0 {
            Err(ParseError::InvalidModulatorChunkSize(size))
        } else {
            let amount = size / 10;

            let data = pmod.read_contents(file).unwrap();
            let mut reader = Reader::new(data);

            (0..amount).map(|_| Self::read(&mut reader)).collect()
        }
    }
}

/// 8.4  Default Modulators
mod default_modulators {
    // TODO: default_modulators

    // use super::*;
    // use crate::data::generator::GeneratorType;

    // /// 8.4.1  MIDI Note-On Velocity to Initial Attenuation
    // static DEFAULT_VEL2ATT_MOD: Modulator = Modulator {
    //     dest: GeneratorType::InitialAttenuation,
    //     amount: 960,

    //     src1: 2,
    //     flags1: MOD_GC | MOD_CONCAVE | MOD_UNIPOLAR | MOD_NEGATIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.2  MIDI Note-On Velocity to Filter Cutoff
    // static DEFAULT_VEL2FILTER_MOD: Modulator = Modulator {
    //     dest: GeneratorType::InitialFilterFc,
    //     amount: -2400,

    //     src1: 2,
    //     flags1: MOD_GC | MOD_LINEAR | MOD_UNIPOLAR | MOD_NEGATIVE,

    //     src2: 2,
    //     flags2: MOD_GC | MOD_SWITCH | MOD_UNIPOLAR | MOD_POSITIVE,
    // };

    // /// 8.4.3  MIDI Channel Pressure to Vibrato LFO Pitch Depth
    // static DEFAULT_AT2VIBLFO_MOD: Modulator = Modulator {
    //     dest: GeneratorType::VibLfoToPitch,
    //     amount: 50,

    //     src1: 13,
    //     flags1: MOD_GC | MOD_LINEAR | MOD_UNIPOLAR | MOD_POSITIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.4  MIDI Continuous Controller 1 to Vibrato LFO Pitch Depth
    // static DEFAULT_MOD2VIBLFO_MOD: Modulator = Modulator {
    //     dest: GeneratorType::VibLfoToPitch,
    //     amount: 50,

    //     src1: 1,
    //     flags1: MOD_CC | MOD_LINEAR | MOD_UNIPOLAR | MOD_POSITIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.5  MIDI Continuous Controller 7 to Initial Attenuation
    // static DEFAULT_ATT_MOD: Modulator = Modulator {
    //     dest: GeneratorType::InitialAttenuation,
    //     amount: 960,

    //     src1: 7,
    //     flags1: MOD_CC | MOD_CONCAVE | MOD_UNIPOLAR | MOD_NEGATIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.6  MIDI Continuous Controller 10 to Pan Position
    // static DEFAULT_PAN_MOD: Modulator = Modulator {
    //     amount: 500,
    //     dest: GeneratorType::Pan,

    //     src1: 10,
    //     flags1: MOD_CC | MOD_LINEAR | MOD_BIPOLAR | MOD_POSITIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.7  MIDI Continuous Controller 11 to Initial Attenuation
    // static DEFAULT_EXPR_MOD: Modulator = Modulator {
    //     amount: 960,
    //     dest: GeneratorType::InitialAttenuation,

    //     src1: 11,
    //     flags1: MOD_CC | MOD_CONCAVE | MOD_UNIPOLAR | MOD_NEGATIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.8  MIDI Continuous Controller 91 to Reverb Effects Send
    // static DEFAULT_REVERB_MOD: Modulator = Modulator {
    //     amount: 200,
    //     dest: GeneratorType::ReverbEffectsSend,

    //     src1: 91,
    //     flags1: MOD_CC | MOD_LINEAR | MOD_UNIPOLAR | MOD_POSITIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.9  MIDI Continuous Controller 93 to Chorus Effects Send
    // static DEFAULT_CHORUS_MOD: Modulator = Modulator {
    //     amount: 200,
    //     dest: GeneratorType::ChorusSend,

    //     src1: 93,
    //     flags1: MOD_CC | MOD_LINEAR | MOD_UNIPOLAR | MOD_POSITIVE,

    //     src2: 0,
    //     flags2: 0,
    // };

    // /// 8.4.10  MIDI Pitch Wheel to Initial Pitch Controlled by MIDI Pitch Wheel Sensitivity
    // static DEFAULT_PITCH_BEND_MOD: Modulator = Modulator {
    //     amount: 12700,
    //     dest: GeneratorType::Pitch,

    //     src1: 14, // PITCHWHEEL
    //     flags1: MOD_GC | MOD_LINEAR | MOD_BIPOLAR | MOD_POSITIVE,

    //     src2: 16, // PITCHWHEELSENS
    //     flags2: MOD_GC | MOD_LINEAR | MOD_UNIPOLAR | MOD_POSITIVE,
    // };
}
