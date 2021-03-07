use super::super::utils::Reader;
use riff::Chunk;

use std::io::{Read, Seek};

#[derive(Debug, Clone)]
pub struct SFPresetHeader {
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

impl SFPresetHeader {
    pub fn read(reader: &mut Reader) -> Self {
        let name: String = reader.read_string(20).trim_end().to_owned();
        let preset: u16 = reader.read_u16();
        let bank: u16 = reader.read_u16();
        let bag_id: u16 = reader.read_u16();

        let library: u32 = reader.read_u32();
        let genre: u32 = reader.read_u32();
        let morphology: u32 = reader.read_u32();

        Self {
            name,
            preset,
            bank,
            bag_id,
            library,
            genre,
            morphology,
        }
    }

    pub fn read_all<F: Read + Seek>(phdr: &Chunk, file: &mut F) -> Vec<Self> {
        assert_eq!(phdr.id().as_str(), "phdr");

        let size = phdr.len();
        if size % 38 != 0 || size == 0 {
            panic!("Preset header chunk size is invalid");
        }

        let amount = size / 38;

        let data = phdr.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}
