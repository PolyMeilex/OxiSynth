// use crate::{engine, Result};
// use bitflags::bitflags;
// use std::{
//     marker::PhantomData,
//     ops::{Bound, RangeBounds},
// };

pub use crate::engine::settings::Settings;

#[cfg(test)]
mod test {
    //     use super::*;

    //     #[test]
    //     fn settings() {
    //         let settings = Settings::new().unwrap();

    //         drop(settings);
    //     }

    //     #[test]
    //     fn num_setting() {
    //         let mut settings = Settings::new().unwrap();
    //         let gain = settings.num("synth.gain").unwrap();

    //         assert_eq!(gain.default(), 0.2f32 as f64);
    //         //assert_eq!(gain.range().min, Some(0.0));
    //         //assert_eq!(gain.range().max, Some(10.0));

    //         assert_eq!(gain.get(), Some(0.2f32 as f64));
    //         assert!(gain.set(0.5));
    //         assert_eq!(gain.get(), Some(0.5));
    //     }

    //     #[test]
    //     fn int_setting() {
    //         let mut settings = Settings::new().unwrap();
    //         let polyphony = settings.int("synth.polyphony").unwrap();

    //         assert_eq!(polyphony.default(), 256);
    //         //assert_eq!(polyphony.range().min, Some(1));
    //         //assert_eq!(polyphony.range().max, Some(65535));

    //         assert_eq!(polyphony.get(), Some(256));
    //         assert!(polyphony.set(512));
    //         assert_eq!(polyphony.get(), Some(512));
    //     }

    //     #[test]
    //     fn str_setting() {
    //         let mut settings = Settings::new().unwrap();
    //         let active = settings.str_("synth.drums-channel.active").unwrap();

    //         assert_eq!(active.default(), "yes");

    //         assert_eq!(active.get(), Some("yes".to_string()));
    //         assert!(active.set("no"));
    //         assert_eq!(active.get(), Some("no".to_string()));
    //     }
}
