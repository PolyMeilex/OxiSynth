use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as StdResult,
};

/// Channel number
pub type Chan = u32;

/// Key number
pub type Key = u32;

/// Velocity value
pub type Vel = u32;

/// Control number
pub type Ctrl = u32;

/// Control value
pub type Val = u32;

/// Program number (`0..=127`)
pub type Prog = u32;

/// Bank number (`0..=127`)
pub type Bank = u32;

/// Font Id
pub type FontId = u32;

/// Preset Id
pub type PresetId = u32;

/// Generic result type
pub type Result<T> = StdResult<T, Error>;

/// Result without value
pub type Status = Result<()>;

/// Common error type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    Alloc,
    Fluid(String),
    Path,
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::Error::*;
        match self {
            Alloc => "Allocation error".fmt(f),
            Fluid(error) => {
                "Fluidlite error: ".fmt(f)?;
                error.fmt(f)
            }
            Path => "Invalid path".fmt(f),
        }
    }
}

pub(crate) fn result_from_ptr<T>(ptr: *mut T) -> Result<*mut T> {
    if ptr.is_null() {
        Err(Error::Alloc)
    } else {
        Ok(ptr)
    }
}

pub(crate) fn option_from_ptr<T>(ptr: *mut T) -> Option<*mut T> {
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}
