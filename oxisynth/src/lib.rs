mod arena;

pub use oxisynth_chorus as chorus;
pub use oxisynth_reverb as reverb;

mod core;
mod error;
mod midi_event;
pub mod settings;
mod synth;
mod tuning;

mod unsafe_stuff;

pub use crate::core::soundfont::{Preset, SoundFont};
pub use error::OxiError;
pub use midi_event::MidiEvent;
pub use tuning::{Tuning, TuningManager};

pub use crate::arena::Index;
pub type SoundFontId = Index<SoundFont>;

pub use settings::*;
pub use synth::*;
