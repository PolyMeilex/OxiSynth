#![forbid(unsafe_code)]

#[macro_use]
extern crate lazy_static;

pub mod chorus;
pub mod reverb;

pub mod settings;
pub use settings::{Settings, SettingsError, SynthDescriptor};

pub mod tuning;
pub use tuning::Tuning;

pub mod synth;
pub use synth::{bank, bank::BankOffset, generator, InterpolationMethod, Synth};

mod conv;

pub mod soundfont;
pub use self::soundfont::{SoundFont, SoundFontId};
