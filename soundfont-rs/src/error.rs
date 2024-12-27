use std::array::TryFromSliceError;
use std::io;
use std::str::Utf8Error;

use crate::riff::Chunk;

#[allow(unused)]
type Result<T> = std::result::Result<T, self::Error>;

#[derive(Debug)]
pub enum Error {
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

    MissingChunk(MissingChunk),
}

#[derive(Debug)]
pub enum MissingChunk {
    /// "INFO"
    Info,
    /// "sdta"
    SampleData,
    /// "pdta"
    Hydra,
    /// "ifil"
    Version,

    /// "phdr"
    PresetHeaders,
    /// "pbag"
    PresetBags,
    /// "pmod"
    PresetModulators,
    /// "pgen"
    PresetGenerators,

    /// "inst"
    InstrumentHeaders,
    /// "ibag"
    InstrumentBags,
    /// "imod"
    InstrumentModulators,
    /// "igen"
    InstrumentGenerators,

    /// "shdr"
    SampleHeaders,
}

// TODO: Proper error, maybe with `thiserror`
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl From<MissingChunk> for Error {
    fn from(err: MissingChunk) -> Self {
        Self::MissingChunk(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::StringError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<TryFromSliceError> for Error {
    fn from(err: TryFromSliceError) -> Self {
        Self::NumSliceError(err)
    }
}
