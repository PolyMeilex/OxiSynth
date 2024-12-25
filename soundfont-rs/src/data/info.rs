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

        let children: Vec<_> = info.iter(file).collect();

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

        for ch in children.into_iter() {
            let ch = ch?;
            let id = ch.id();

            match id {
                // Refers to the version of the Sound Font RIFF file
                ChunkId::ifil => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    version = Some(Version {
                        major: data.read_u16()?,
                        minor: data.read_u16()?,
                    });
                }
                // Refers to the target Sound Engine
                ChunkId::isng => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    sound_engine = Some(data.read_string(ch.len() as usize)?);
                }
                // Refers to the Sound Font Bank Name
                ChunkId::INAM => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    bank_name = Some(data.read_string(ch.len() as usize)?);
                }
                // Refers to the Sound ROM Name
                ChunkId::irom => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    rom_name = Some(data.read_string(ch.len() as usize)?);
                }
                // Refers to the Sound ROM Version
                ChunkId::iver => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    rom_version = Some(Version {
                        major: data.read_u16()?,
                        minor: data.read_u16()?,
                    });
                }
                // Refers to the Date of Creation of the Bank
                ChunkId::ICRD => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    creation_date = Some(data.read_string(ch.len() as usize)?);
                }
                // Sound Designers and Engineers for the Bank
                ChunkId::IENG => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    engineers = Some(data.read_string(ch.len() as usize)?);
                }
                // Product for which the Bank was intended
                ChunkId::IPRD => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    product = Some(data.read_string(ch.len() as usize)?);
                }
                // Contains any Copyright message
                ChunkId::ICOP => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    copyright = Some(data.read_string(ch.len() as usize)?);
                }
                // Contains any Comments on the Bank
                ChunkId::ICMT => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    comments = Some(data.read_string(ch.len() as usize)?);
                }
                // The SoundFont tools used to create and alter the bank
                ChunkId::ISFT => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
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
