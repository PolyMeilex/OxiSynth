use wasm_bindgen::prelude::*;
use web_sys::console;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use std::sync::mpsc::{self, Receiver, Sender};

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    Ok(())
}

enum MidiEvent {
    NoteOn { ch: u8, key: u8, vel: u8 },
    NoteOff { ch: u8, key: u8 },
}

#[wasm_bindgen]
pub struct Handle(Stream, Sender<MidiEvent>);

impl Handle {
    fn note_on(&mut self, ch: u8, key: u8, vel: u8) {
        self.1.send(MidiEvent::NoteOn { ch, key, vel }).ok();
    }
    fn note_off(&mut self, ch: u8, key: u8) {
        self.1.send(MidiEvent::NoteOff { ch, key }).ok();
    }
}

#[wasm_bindgen]
pub struct Synth(oxisynth::Synth);

#[wasm_bindgen]
pub fn noteOn(h: &mut Handle, note: i32) {
    h.note_on(0, note as _, 100);
}

#[wasm_bindgen]
pub fn beep() -> Handle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<MidiEvent>();
    Handle(
        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), rx),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), rx),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), rx),
        },
        tx,
    )
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, rx: Receiver<MidiEvent>) -> Stream
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut synth = {
        let mut settings = oxisynth::Settings::default();

        settings.synth.sample_rate = sample_rate;

        let mut synth = oxisynth::Synth::new(settings);

        // Load from memory
        use std::io::Cursor;
        let mut file = Cursor::new(include_bytes!("../../../testdata/Boomwhacker.sf2"));

        synth.sfload(&mut file, true).unwrap();
        synth.set_sample_rate(sample_rate);
        synth.set_gain(1.0);

        synth
    };

    let mut next_value = move || {
        let (l, r) = synth.read_next();

        if let Ok(e) = rx.try_recv() {
            match e {
                MidiEvent::NoteOn { ch, key, vel } => {
                    synth.note_on(ch, key, vel).ok();
                }
                MidiEvent::NoteOff { ch, key } => {
                    synth.note_off(ch, key).ok();
                }
            }
        }

        l
    };

    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_data(data, channels, &mut next_value),
            err_fn,
        )
        .unwrap();
    stream.play().unwrap();
    stream
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
