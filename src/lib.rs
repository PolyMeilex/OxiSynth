pub(crate) use core as oxi;

mod settings;
mod synth;

pub use core::soundfont::{Preset, SoundFont};

pub use core::TypedIndex;
pub type SoundFontId = TypedIndex<SoundFont>;

pub use self::settings::*;
pub use self::synth::*;
