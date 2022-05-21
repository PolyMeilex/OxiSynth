#![forbid(unsafe_code)]

pub mod chorus;
pub mod reverb;

pub mod settings;
pub use settings::{Settings, SettingsError, SynthDescriptor};

pub mod tuning;
pub use tuning::{Tuning, TuningManager};

pub mod synth;
pub use synth::{font_bank, InterpolationMethod, Synth};

pub use synth::soundfont::{self, SoundFont};

mod utils;
pub use utils::TypedIndex;

pub mod error;
pub use error::OxiError;

pub mod midi_event;
pub use midi_event::MidiEvent;
