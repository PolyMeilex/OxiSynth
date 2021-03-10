use std::array::TryFromSliceError;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum ParseError {
    StringError(Utf8Error),
    NumSliceError(TryFromSliceError),

    InvalidBagChunkSize(u32),
    InvalidGeneratorChunkSize(u32),
    InvalidInstrumentChunkSize(u32),
    InvalidModulatorChunkSize(u32),
    InvalidPresetChunkSize(u32),
    InvalidSampleChunkSize(u32),
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        Self::StringError(err)
    }
}

impl From<TryFromSliceError> for ParseError {
    fn from(err: TryFromSliceError) -> Self {
        Self::NumSliceError(err)
    }
}
