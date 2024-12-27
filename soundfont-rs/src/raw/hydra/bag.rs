use crate::{
    error::Error,
    riff::{ChunkId, ScratchReader},
};

use super::super::utils::Reader;
use crate::riff::Chunk;
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct Bag {
    pub generator_id: u16,
    pub modulator_id: u16,
}

impl Bag {
    pub(crate) fn read(reader: &mut Reader) -> Result<Self, Error> {
        let generator_id: u16 = reader.read_u16()?;
        let modulator_id: u16 = reader.read_u16()?;

        Ok(Self {
            generator_id,
            modulator_id,
        })
    }

    pub(crate) fn read_all(
        pbag: &Chunk,
        file: &mut ScratchReader<impl Read + Seek>,
    ) -> Result<Vec<Self>, Error> {
        assert!(pbag.id() == ChunkId::pbag || pbag.id() == ChunkId::ibag);

        let size = pbag.len();
        if size % 4 != 0 || size == 0 {
            Err(Error::InvalidBagChunkSize(size))
        } else {
            let amount = size / 4;

            let data = pbag.read_contents(file)?;
            let mut reader = Reader::new(data);

            (0..amount).map(|_| Self::read(&mut reader)).collect()
        }
    }
}
