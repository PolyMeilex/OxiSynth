#![forbid(unsafe_code)]

pub mod synth;
pub use synth::{font_bank, InterpolationMethod, Synth};

pub use synth::soundfont::{self, SoundFont};
