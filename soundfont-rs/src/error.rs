use std::array::TryFromSliceError;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum ParseError {
    StringError(Utf8Error),
    NumSliceError(TryFromSliceError),
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
