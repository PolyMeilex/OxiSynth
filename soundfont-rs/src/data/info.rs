use super::utils::Reader;
use crate::riff::Chunk;
use crate::{error::ParseError, riff::ChunkId};

use std::io::{Read, Seek};

#[derive(Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

/// Supplemental Information
#[derive(Debug)]
pub struct Info {
    /// Refers to the version of the Sound Font RIFF file
    pub version: Version,
    /// Refers to the target Sound Engine
    pub sound_engine: String,
    /// Refers to the Sound Font Bank Name
    pub bank_name: String,

    /// Refers to the Sound ROM Name
    pub rom_name: Option<String>,
    /// Refers to the Sound ROM Version
    pub rom_version: Option<Version>,

    /// Refers to the Date of Creation of the Bank
    pub creation_date: Option<String>,
    /// Sound Designers and Engineers for the Bank
    pub engineers: Option<String>,
    /// Product for which the Bank was intended
    pub product: Option<String>,
    /// Contains any Copyright message
    pub copyright: Option<String>,
    /// Contains any Comments on the Bank
    pub comments: Option<String>,
    /// The SoundFont tools used to create and alter the bank
    pub software: Option<String>,
}

impl Info {
    pub fn read<F: Read + Seek>(info: &Chunk, file: &mut F) -> Result<Self, ParseError> {
        assert_eq!(info.id(), ChunkId::LIST);
        assert_eq!(info.read_type(file)?, ChunkId::INFO);

        let mut version = None;
        let mut sound_engine = None;
        let mut bank_name = None;

        let mut rom_name = None;
        let mut rom_version = None;

        let mut creation_date = None;
        let mut engineers = None;
        let mut product = None;
        let mut copyright = None;
        let mut comments = None;
        let mut software = None;

        let mut buf: Vec<u8> = Vec::new();
        let mut iter = info.iter();
        while let Some(ch) = iter.next(file) {
            let ch = ch?;
            let id = ch.id();

            match id {
                // Refers to the version of the Sound Font RIFF file
                ChunkId::ifil => {
                    let mut data = [0u8; 16 * 2];
                    ch.read_to(file, &mut data).unwrap();

                    version = Some(Version {
                        major: u16::from_le_bytes([data[0], data[1]]),
                        minor: u16::from_le_bytes([data[2], data[3]]),
                    });
                }
                // Refers to the target Sound Engine
                ChunkId::isng => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    sound_engine = Some(data.read_string(ch.len() as usize)?);
                }
                // Refers to the Sound Font Bank Name
                ChunkId::INAM => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    bank_name = Some(data.read_string(ch.len() as usize)?);
                }
                // Refers to the Sound ROM Name
                ChunkId::irom => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    rom_name = Some(data.read_string(ch.len() as usize)?);
                }
                // Refers to the Sound ROM Version
                ChunkId::iver => {
                    let mut data = [0u8; 16 * 2];
                    ch.read_to(file, &mut data).unwrap();

                    rom_version = Some(Version {
                        major: u16::from_le_bytes([data[0], data[1]]),
                        minor: u16::from_le_bytes([data[2], data[3]]),
                    });
                }
                // Refers to the Date of Creation of the Bank
                ChunkId::ICRD => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    creation_date = Some(data.read_string(ch.len() as usize)?);
                }
                // Sound Designers and Engineers for the Bank
                ChunkId::IENG => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    engineers = Some(data.read_string(ch.len() as usize)?);
                }
                // Product for which the Bank was intended
                ChunkId::IPRD => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    product = Some(data.read_string(ch.len() as usize)?);
                }
                // Contains any Copyright message
                ChunkId::ICOP => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    copyright = Some(data.read_string(ch.len() as usize)?);
                }
                // Contains any Comments on the Bank
                ChunkId::ICMT => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    comments = Some(data.read_string(ch.len() as usize)?);
                }
                // The SoundFont tools used to create and alter the bank
                ChunkId::ISFT => {
                    let data = ch.read_to_scratch(file, &mut buf)?;
                    let mut data = Reader::new(data);
                    software = Some(data.read_string(ch.len() as usize)?);
                }
                _ => {
                    return Err(ParseError::UnexpectedMemeberOfInfo(ch));
                }
            }
        }

        Ok(Info {
            version: version.unwrap(),
            // Those two are requited by the specs, but you can often find files without them
            // so that's why `unwrap_or_default` is used.
            sound_engine: sound_engine.unwrap_or_default(),
            bank_name: bank_name.unwrap_or_default(),

            rom_name,
            rom_version,

            creation_date,
            engineers,
            product,
            copyright,
            comments,
            software,
        })
    }
}
