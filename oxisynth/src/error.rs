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
        };

        Ok(())
    }
}
