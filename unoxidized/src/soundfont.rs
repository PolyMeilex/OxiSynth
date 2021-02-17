use std::any::Any;

use crate::fileapi::FileSystem;

use super::synth::Synth;

pub struct SoundFontLoader {
    pub data: *mut libc::c_void,
    pub free: Option<unsafe fn(_: *mut SoundFontLoader) -> i32>,
    pub load: Option<unsafe fn(_: *mut SoundFontLoader, _: &[u8]) -> Option<SoundFont>>,
    pub filesystem: Box<dyn FileSystem>,
}

#[derive(Copy, Clone)]
pub struct Preset {
    pub data: *mut libc::c_void,
    pub sfont: *const SoundFont,
    pub free: Option<unsafe fn(_: *mut Preset) -> i32>,
    pub get_name: Option<unsafe fn(_: *const Preset) -> Vec<u8>>,
    pub get_banknum: Option<unsafe fn(_: *const Preset) -> i32>,
    pub get_num: Option<unsafe fn(_: *const Preset) -> i32>,
    pub noteon: Option<unsafe fn(_: *mut Preset, _: &mut Synth, _: i32, _: i32, _: i32) -> i32>,
}

pub struct SoundFont {
    pub data: Box<dyn Any>,
    pub id: u32,
    pub free: Option<unsafe fn(_: *mut SoundFont) -> i32>,
    pub get_name: Option<unsafe fn(_: *const SoundFont) -> Option<Vec<u8>>>,
    pub get_preset: Option<unsafe fn(_: *const SoundFont, _: u32, _: u32) -> *mut Preset>,
    pub iteration_start: Option<unsafe fn(_: *mut SoundFont) -> ()>,
    pub iteration_next: Option<unsafe fn(_: *mut SoundFont, _: *mut Preset) -> i32>,
}

#[derive(Copy, Clone)]
pub struct Sample {
    pub name: [u8; 21],
    pub start: u32,
    pub end: u32,
    pub loopstart: u32,
    pub loopend: u32,
    pub samplerate: u32,
    pub origpitch: i32,
    pub pitchadj: i32,
    pub sampletype: i32,
    pub valid: i32,
    pub data: *mut i16,
    pub amplitude_that_reaches_noise_floor_is_valid: i32,
    pub amplitude_that_reaches_noise_floor: f64,
    pub refcount: u32,
}
