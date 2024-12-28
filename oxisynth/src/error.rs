use crate::SoundFontId;

pub(crate) fn range_check<E, T: PartialOrd, C: std::ops::RangeBounds<T>>(
    range: C,
    value: &T,
    error: E,
) -> Result<(), E> {
    if range.contains(value) {
        Ok(())
    } else {
        Err(error)
    }
}

#[derive(Debug)]
pub enum LoadError {
    Parsing(soundfont::Error),
    Version {
        version: soundfont::raw::Version,
        max: u16,
    },
    Io(std::io::Error),
    SampleNotFound {
        name: String,
    },
}

impl From<soundfont::Error> for LoadError {
    fn from(err: soundfont::Error) -> Self {
        Self::Parsing(err)
    }
}

impl From<std::io::Error> for LoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::error::Error for LoadError {}
impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing(error) => {
                write!(f, "SoundFont parsing error: {error}")?;
            }
            Self::Version { version, max } => {
                write!(f, "Unsupported version: {version:?}, max supported {max}")?;
            }
            Self::Io(error) => {
                write!(f, "IO error while reading SoundFont: {error}")?;
            }
            Self::SampleNotFound { name } => {
                write!(f, "Sample {name:?} not found")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum OxiError {
    KeyOutOfRange,
    VelocityOutOfRange,
    ChannelOutOfRange,
    CtrlOutOfRange,
    CCValueOutOfRange,
    ProgramOutOfRange,
    KeyPressureOutOfRange,
    ChannelPressureOutOfRange,
    PithBendOutOfRange,
    ChannelHasNoPreset,
    PresetNotFound {
        bank_id: u32,
        preset_id: u8,
        sfont_id: SoundFontId,
    },
    InvalidPolyphony,
}

impl std::error::Error for OxiError {}

impl std::fmt::Display for OxiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OxiError::KeyOutOfRange => {
                write!(f, "Key out of range (0-127)")?;
            }
            OxiError::VelocityOutOfRange => {
                write!(f, "Velocity out of range (0-127)")?;
            }
            OxiError::ChannelOutOfRange => {
                write!(f, "Channel out of range")?;
            }
            OxiError::CtrlOutOfRange => {
                write!(f, "Ctrl out of range (0-127)")?;
            }
            OxiError::CCValueOutOfRange => {
                write!(f, "CC Value out of range (0-127)")?;
            }
            OxiError::ProgramOutOfRange => {
                write!(f, "Program out of range")?;
            }
            OxiError::KeyPressureOutOfRange => {
                write!(f, "Key pressure out of range (0-127)")?;
            }
            OxiError::ChannelPressureOutOfRange => {
                write!(f, "Channel pressure out of range (0-127)")?;
            }
            OxiError::PithBendOutOfRange => {
                write!(f, "PithBend out of range")?;
            }
            OxiError::ChannelHasNoPreset => {
                write!(f, "Channel has no preset")?;
            }
            OxiError::PresetNotFound {
                bank_id,
                preset_id,
                sfont_id,
            } => {
                write!(f,"There is no preset with bank number {bank_id} and preset number {preset_id} in SoundFont {sfont_id:?}")?;
            }
            OxiError::InvalidPolyphony => {
                write!(f, "Only polyphony >= 1 is allowed")?;
            }
        };

        Ok(())
    }
}
