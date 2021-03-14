#![forbid(unsafe_code)]

#[macro_use]
extern crate lazy_static;

pub mod chorus;
pub mod reverb;

mod settings;
pub use settings::{Settings, SettingsError, SynthDescriptor};

pub mod tuning;
pub use tuning::Tuning;

pub mod synth;
pub use synth::{generator, InterpolationMethod, SoundFont, Synth};

mod conv;
