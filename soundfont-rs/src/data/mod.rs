mod utils;

pub mod hydra;
pub mod info;
pub mod sample_data;

use crate::error::ParseError;
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
        let sfbk = riff::Chunk::read(file, 0).unwrap();
        assert_eq!(sfbk.id().as_str(), "RIFF");
        assert_eq!(sfbk.read_type(file).unwrap().as_str(), "sfbk");

        let chunks: Vec<_> = sfbk.iter(file).collect();

        let mut info = None;
        let mut sample_data = None;
        let mut hydra = None;

        for ch in chunks.into_iter() {
            let ch = ch?;
            assert_eq!(ch.id().as_str(), "LIST");
            let ty = ch.read_type(file).unwrap();
            match ty.as_str() {
                "INFO" => {
                    info = Some(Info::read(&ch, file)?);
                }
                "sdta" => {
                    sample_data = Some(SampleData::read(&ch, file)?);
                }
                "pdta" => {
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
