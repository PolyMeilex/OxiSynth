#[macro_use]
mod redoxsynth_macros {
    macro_rules! fluid_log {
        ($log_level:expr, $fmt_string:expr, $( $arg:expr ),*) => {
            println!($fmt_string, $( $arg ),*);
        }
    }

    macro_rules! gerr {
        ($err:expr, $fmt_string:expr, $( $arg:expr ),*) => {
            { println!($fmt_string, $( $arg ),*); 0 }
        }
    }
}

pub(crate) mod channel;
pub(crate) mod chorus;
pub(crate) mod conv;
pub(crate) mod dsp_float;
pub(crate) mod gen;
pub(crate) mod modulator;
pub(crate) mod reverb;
pub(crate) mod settings;
pub(crate) mod sfloader;
pub(crate) mod soundfont;
pub(crate) mod synth;
pub(crate) mod tuning;
pub(crate) mod voice;
