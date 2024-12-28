use crate::SoundFontId;

#[derive(thiserror::Error, Debug)]
pub enum OxiError {
    #[error("Key out of range (0-127)")]
    KeyOutOfRange,
    #[error("Velocity out of range (0-127)")]
    VelocityOutOfRange,
    #[error("Channel out of range")]
    ChannelOutOfRange,
    #[error("Ctrl out of range (0-127)")]
    CtrlOutOfRange,
    #[error("CC Value out of range (0-127)")]
    CCValueOutOfRange,
    #[error("Program out of range")]
    ProgramOutOfRange,
    #[error("Key pressure out of range (0-127)")]
    KeyPressureOutOfRange,
    #[error("Channel pressure out of range (0-127)")]
    ChannelPressureOutOfRange,
    #[error("PithBend out of range")]
    PithBendOutOfRange,
    #[error("Channel has no preset")]
    ChannelHasNoPreset,
    #[error(
        "There is no preset with bank number {bank_id} and preset number {preset_id} in SoundFont {sfont_id:?}"
    )]
    PresetNotFound {
        bank_id: u32,
        preset_id: u8,
        sfont_id: SoundFontId,
    },
}
