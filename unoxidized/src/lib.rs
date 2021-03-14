#![forbid(unsafe_code)]

#[macro_use]
extern crate lazy_static;

pub mod chorus;
pub mod reverb;

mod conv;
pub mod settings;
pub mod synth;
pub mod tuning;

pub use synth::channel;
pub use synth::generator;
pub use synth::modulator;
pub use synth::soundfont;
pub use synth::voice_pool;
