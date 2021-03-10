use super::super::utils::Reader;
use crate::error::ParseError;
use riff::Chunk;
use std::io::{Read, Seek};

#[derive(Debug, Clone)]
pub struct SFInstrumentHeader {
    pub name: String,
    pub bag_id: u16,
}

impl SFInstrumentHeader {
    pub fn read(reader: &mut Reader) -> Result<Self, ParseError> {
        let name: String = reader.read_string(20)?.trim_end().to_owned();
        let bag_id: u16 = reader.read_u16()?;

        Ok(Self { name, bag_id })
    }

    pub fn read_all<F: Read + Seek>(phdr: &Chunk, file: &mut F) -> Result<Vec<Self>, ParseError> {
        assert_eq!(phdr.id().as_str(), "inst");

        let size = phdr.len();
        if size % 22 != 0 || size == 0 {
            panic!("Instrument header chunk size is invalid");
        }

        let amount = size / 22;

        let data = phdr.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}
