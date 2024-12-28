mod arena;

pub use oxisynth_chorus as chorus;
pub use oxisynth_reverb as reverb;

mod core;
mod error;
mod midi_event;
mod range_check;
pub mod settings;
mod synth;
mod tuning;

pub use crate::core::soundfont::{Preset, SoundFont};
pub use error::OxiError;
pub use midi_event::MidiEvent;
pub use tuning::{Tuning, TuningManager};

pub use crate::arena::TypedIndex;
pub type SoundFontId = TypedIndex<SoundFont>;

pub use settings::*;
pub use synth::*;
