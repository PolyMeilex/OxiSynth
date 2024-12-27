use super::super::utils::Reader;
use crate::riff::{Chunk, ScratchReader};
use crate::{error::Error, riff::ChunkId};
use std::io::{Read, Seek};

#[derive(Debug, Clone)]
pub struct InstrumentHeader {
    pub name: String,
    pub bag_id: u16,
}

impl InstrumentHeader {
    pub(crate) fn read(reader: &mut Reader) -> Result<Self, Error> {
        let name: String = reader.read_string(20)?.trim_end().to_owned();
        let bag_id: u16 = reader.read_u16()?;

        Ok(Self { name, bag_id })
    }

    pub(crate) fn read_all(
        phdr: &Chunk,
        file: &mut ScratchReader<impl Read + Seek>,
    ) -> Result<Vec<Self>, Error> {
        assert_eq!(phdr.id(), ChunkId::inst);

        let size = phdr.len();
        if size % 22 != 0 || size == 0 {
            Err(Error::InvalidInstrumentChunkSize(size))
        } else {
            let amount = size / 22;

            let data = phdr.read_contents(file)?;
            let mut reader = Reader::new(data);

            (0..amount).map(|_| Self::read(&mut reader)).collect()
        }
    }
}
