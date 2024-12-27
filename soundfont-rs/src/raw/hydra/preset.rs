use super::super::utils::Reader;
use crate::riff::{Chunk, ScratchReader};
use crate::{error::Error, riff::ChunkId};

use std::io::{Read, Seek};

#[derive(Debug, Clone)]
pub struct PresetHeader {
    /// The name of the preset
    pub name: String,
    /// The MIDI preset number which to apply to the preset.
    pub preset: u16,
    /// The preset bank
    pub bank: u16,
    pub bag_id: u16,

    /// Reserved?
    pub library: u32,
    /// Reserved?
    pub genre: u32,
    /// Reserved?
    pub morphology: u32,
}

impl PresetHeader {
    pub(crate) fn read(reader: &mut Reader) -> Result<Self, Error> {
        let name: String = reader.read_string(20)?.trim_end().to_owned();
        let preset: u16 = reader.read_u16()?;
        let bank: u16 = reader.read_u16()?;
        let bag_id: u16 = reader.read_u16()?;

        let library: u32 = reader.read_u32()?;
        let genre: u32 = reader.read_u32()?;
        let morphology: u32 = reader.read_u32()?;

        Ok(Self {
            name,
            preset,
            bank,
            bag_id,
            library,
            genre,
            morphology,
        })
    }

    pub(crate) fn read_all(
        phdr: &Chunk,
        file: &mut ScratchReader<impl Read + Seek>,
    ) -> Result<Vec<Self>, Error> {
        assert_eq!(phdr.id(), ChunkId::phdr);

        let size = phdr.len();
        if size % 38 != 0 || size == 0 {
            Err(Error::InvalidPresetChunkSize(size))
        } else {
            let amount = size / 38;

            let data = phdr.read_contents(file)?;
            let mut reader = Reader::new(data);

            (0..amount).map(|_| Self::read(&mut reader)).collect()
        }
    }
}
