pub mod chorus;
mod core;
pub mod reverb;
mod synth;

pub use crate::core::soundfont::{Preset, SoundFont};
pub use crate::core::{MidiEvent, OxiError};

pub use crate::core::TypedIndex;
pub type SoundFontId = TypedIndex<SoundFont>;

pub use self::settings::*;
pub use self::synth::*;

pub mod settings {
    pub use crate::core::{Settings, SettingsError, SynthDescriptor};
}

#[macro_use]
extern crate lazy_static;
