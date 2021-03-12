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
