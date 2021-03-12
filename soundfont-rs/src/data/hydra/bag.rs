use crate::error::ParseError;

use super::super::utils::Reader;
use riff::Chunk;
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct Bag {
    pub generator_id: u16,
    pub modulator_id: u16,
}

impl Bag {
    pub fn read(reader: &mut Reader) -> Result<Self, ParseError> {
        let generator_id: u16 = reader.read_u16()?;
        let modulator_id: u16 = reader.read_u16()?;

        Ok(Self {
            generator_id,
            modulator_id,
        })
    }

    pub fn read_all<F: Read + Seek>(pbag: &Chunk, file: &mut F) -> Result<Vec<Self>, ParseError> {
        assert!(pbag.id().as_str() == "pbag" || pbag.id().as_str() == "ibag");

        let size = pbag.len();
        if size % 4 != 0 || size == 0 {
            Err(ParseError::InvalidBagChunkSize(size))
        } else {
            let amount = size / 4;

            let data = pbag.read_contents(file).unwrap();
            let mut reader = Reader::new(data);

            (0..amount).map(|_| Self::read(&mut reader)).collect()
        }
    }
}
