//! A low-level SoundFont™ file reader
//!
//! This performs no postprocessing, it just presents the data from the file as is.
//!
//! [`SFData`] is the entry point to the low-level reader,
//! its data structures should mostly reflect [SoundFont™ 2.04 spec](https://github.com/PolyMeilex/OxiSynth/wiki/SoundFont%E2%84%A2-Specification).
//!
//! For higher level abstraction see [`crate::SoundFont2`].

mod utils;

mod hydra;
mod info;
mod sample_data;

use crate::{
    error::{MissingChunk, ParseError},
    riff::{self, ChunkId},
};
use std::io::{Read, Seek};

pub use hydra::*;
pub use info::*;
pub use sample_data::*;

/// A low-level SoundFont™ file reader
///
/// Read [`module level docs`](self) for more information.
#[derive(Debug)]
pub struct RawSoundFontData {
    pub info: Info,
    pub sample_data: SampleData,
    pub hydra: Hydra,
}

impl RawSoundFontData {
    pub fn load<F: Read + Seek>(file: &mut F) -> Result<Self, ParseError> {
        let sfbk = riff::Chunk::read(file, 0)?;
        assert_eq!(sfbk.id(), ChunkId::RIFF);
        assert_eq!(sfbk.read_type(file)?, ChunkId::sfbk);

        let mut info = None;
        let mut sample_data = None;
        let mut hydra = None;

        let mut file = riff::ScratchReader::new(file);
        let mut iter = sfbk.iter();
        while let Some(ch) = iter.next(&mut file) {
            let ch = ch?;
            assert_eq!(ch.id(), ChunkId::LIST);

            match ch.read_type(&mut file)? {
                ChunkId::INFO => {
                    info = Some(Info::read(&ch, &mut file)?);
                }
                ChunkId::sdta => {
                    sample_data = Some(SampleData::read(&ch, &mut file)?);
                }
                ChunkId::pdta => {
                    hydra = Some(Hydra::read(&ch, &mut file)?);
                }
                _ => {
                    return Err(ParseError::UnexpectedMemberOfRoot(ch));
                }
            }
        }

        Ok(RawSoundFontData {
            info: info.ok_or(MissingChunk::Info)?,
            sample_data: sample_data.ok_or(MissingChunk::SampleData)?,
            hydra: hydra.ok_or(MissingChunk::Hydra)?,
        })
    }
}
