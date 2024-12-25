use std::array::TryFromSliceError;
use std::io;
use std::str::Utf8Error;

use crate::riff::Chunk;

#[derive(Debug)]
pub enum ParseError {
    StringError(Utf8Error),
    Io(io::Error),
    NumSliceError(TryFromSliceError),

    InvalidBagChunkSize(u32),
    InvalidGeneratorChunkSize(u32),
    InvalidInstrumentChunkSize(u32),
    InvalidModulatorChunkSize(u32),
    InvalidPresetChunkSize(u32),
    InvalidSampleChunkSize(u32),

    UnknownGeneratorType(u16),
    UnknownSampleType(u16),
    UnknownModulatorTransform(u16),

    UnexpectedMemberOfRoot(Chunk),
    UnexpectedMemberOfHydra(Chunk),
    UnexpectedMemberOfInfo(Chunk),
    UnexpectedMemberOfSampleData(Chunk),
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        Self::StringError(err)
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<TryFromSliceError> for ParseError {
    fn from(err: TryFromSliceError) -> Self {
        Self::NumSliceError(err)
    }
}
