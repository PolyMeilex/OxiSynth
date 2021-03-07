use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{error::Error, path::Path, sync::mpsc::Receiver};

use midir::MidiInput;

const SAMPLES_SIZE: usize = 1410;

enum MidiEvent {
    NoteOn { ch: u8, key: u8, vel: u8 },
    NoteOff { ch: u8, key: u8 },
    Cc { ch: u8, ctrl: u16, val: u16 },
}

pub struct SynthBackend {
    _host: cpal::Host,
    device: cpal::Device,

    stream_config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
}

impl SynthBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or("failed to find a default output device")?;

        let config = device.default_output_config()?;
        let sample_format = config.sample_format();

        let stream_config: cpal::StreamConfig = config.into();

        Ok(Self {
            _host: host,
            device,

            stream_config,
            sample_format,
        })
    }

    fn run<T: cpal::Sample>(&self, rx: Receiver<MidiEvent>, path: &Path) -> cpal::Stream {
        let mut synth = {
            let sample_rate = self.stream_config.sample_rate.0;

            let mut settings = oxisynth::Settings::default();

            settings.synth.sample_rate = sample_rate as f32;

            let mut synth = oxisynth::Synth::new(settings);
            let mut file = std::fs::File::open(path).unwrap();

            synth.sfload(&mut file, true).unwrap();
            synth.set_sample_rate(sample_rate as f32);
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
                    MidiEvent::Cc { ch, ctrl, val } => {
                        synth.cc(ch, ctrl, val);
                    }
                }
            }

            r
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let channels = self.stream_config.channels as usize;

        let stream = self
            .device
            .build_output_stream(
                &self.stream_config,
                move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                    for frame in output.chunks_mut(channels) {
                        let value: T = cpal::Sample::from::<f32>(&next_value());
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();

        stream
    }

    pub fn new_output_connection<P: AsRef<Path>>(
        &mut self,
        path: &P,
    ) -> (cpal::Stream, SynthOutputConnection) {
        let (tx, rx) = std::sync::mpsc::channel::<MidiEvent>();
        let _stream = match self.sample_format {
            cpal::SampleFormat::F32 => self.run::<f32>(rx, path.as_ref()),
            cpal::SampleFormat::I16 => self.run::<i16>(rx, path.as_ref()),
            cpal::SampleFormat::U16 => self.run::<u16>(rx, path.as_ref()),
        };

        (_stream, SynthOutputConnection { tx })
    }
}

pub struct SynthOutputConnection {
    tx: std::sync::mpsc::Sender<MidiEvent>,
}

impl SynthOutputConnection {
    fn note_on(&mut self, ch: u8, key: u8, vel: u8) {
        self.tx.send(MidiEvent::NoteOn { ch, key, vel }).ok();
    }
    fn note_off(&mut self, ch: u8, key: u8) {
        self.tx.send(MidiEvent::NoteOff { ch, key }).ok();
    }
    fn cc(&mut self, ch: u8, ctrl: u16, val: u16) {
        self.tx.send(MidiEvent::Cc { ch, ctrl, val }).ok();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let mut synth = SynthBackend::new()?;

    let mut input = String::new();
    let mut midi_in = MidiInput::new("midir reading input")?;

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();

    use std::io::{stdin, stdout, Write};

    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let (_stream, mut synth_conn) = synth.new_output_connection(&"../../testdata/sin.sf2");

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            if message.len() == 3 {
                let note = message[1];
                if note >= 21 && note <= 108 {
                    if message[0] == 128 || message[2] == 0 {
                        println!("NoteOff {}", message[1]);
                        synth_conn.note_off(0, message[1]);
                        // tx.send((false, message[1], message[2])).unwrap();
                    } else if message[0] == 144 {
                        println!("NoteOn {},{}", message[1], message[2]);
                        synth_conn.note_on(0, message[1], message[2]);
                        // required_notes.lock().unwrap().remove(&note);
                        // tx.send((true, message[1], message[2])).unwrap();
                    }
                }
                if message[0] == 177 {
                    synth_conn.cc(0, message[1] as _, message[2] as _);
                    println!("Sustain {}", if message[2] != 0 { "On" } else { "Off" });
                }
                println!("{}: {:?} (len = {})", stamp, message, message.len());
            }
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );
    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
