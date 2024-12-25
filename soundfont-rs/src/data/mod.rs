mod utils;

pub mod hydra;
pub mod info;
pub mod sample_data;

use crate::{error::ParseError, riff::ChunkId};
use std::io::{Read, Seek};

pub use hydra::*;
pub use info::*;
pub use sample_data::*;

#[derive(Debug)]
pub struct SFData {
    pub info: Info,
    pub sample_data: SampleData,
    pub hydra: Hydra,
}

impl SFData {
    pub fn load<F: Read + Seek>(file: &mut F) -> Result<Self, ParseError> {
        let sfbk = crate::riff::Chunk::read(file, 0).unwrap();
        assert_eq!(sfbk.id(), ChunkId::RIFF);
        assert_eq!(sfbk.read_type(file)?, ChunkId::sfbk);

        let mut info = None;
        let mut sample_data = None;
        let mut hydra = None;

        let mut iter = sfbk.iter();
        while let Some(ch) = iter.next(file) {
            let ch = ch?;
            assert_eq!(ch.id(), ChunkId::LIST);
            match ch.read_type(file)? {
                ChunkId::INFO => {
                    info = Some(Info::read(&ch, file)?);
                }
                ChunkId::sdta => {
                    sample_data = Some(SampleData::read(&ch, file)?);
                }
                ChunkId::pdta => {
                    hydra = Some(Hydra::read(&ch, file)?);
                }
                _ => {
                    return Err(ParseError::UnexpectedMemeberOfRoot(ch));
                }
            }
        }

        Ok(SFData {
            info: info.unwrap(),
            sample_data: sample_data.unwrap(),
            hydra: hydra.unwrap(),
        })
    }
}
