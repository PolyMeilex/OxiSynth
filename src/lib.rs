pub mod chorus;
mod core;
mod error;
mod midi_event;
pub mod reverb;
pub mod settings;
mod synth;
mod tuning;

pub use crate::core::soundfont::{Preset, SoundFont};
pub use error::OxiError;
pub use midi_event::MidiEvent;
pub use tuning::{Tuning, TuningManager};

pub use crate::core::TypedIndex;
pub type SoundFontId = TypedIndex<SoundFont>;

pub use settings::*;
pub use synth::*;

#[macro_use]
extern crate lazy_static;
