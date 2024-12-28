use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{error::Error, path::Path, sync::mpsc::Receiver};

use midir::MidiInput;

use oxisynth::MidiEvent;

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
            let sample_rate = self.stream_config.sample_rate.0 as f32;

            let settings = oxisynth::SynthDescriptor {
                sample_rate,
                gain: 1.0,
                ..Default::default()
            };

            let mut synth = oxisynth::Synth::new(settings).unwrap();
            let mut file = std::fs::File::open(path).unwrap();
            let font = oxisynth::SoundFont::load(&mut file).unwrap();

            synth.add_font(font, true);
            synth.set_sample_rate(sample_rate);
            synth.set_gain(1.0);

            synth
        };

        let mut next_value = move || {
            let (l, r) = synth.read_next();

            if let Ok(e) = rx.try_recv() {
                synth.send_event(e).ok();
            }

            (l, r)
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let channels = self.stream_config.channels as usize;

        let stream = self
            .device
            .build_output_stream(
                &self.stream_config,
                move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                    for frame in output.chunks_mut(channels) {
                        let (l, r) = next_value();

                        let l: T = cpal::Sample::from::<f32>(&l);
                        let r: T = cpal::Sample::from::<f32>(&r);

                        let channels = [l, r];

                        for (id, sample) in frame.iter_mut().enumerate() {
                            *sample = channels[id % 2];
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
    fn note_on(&mut self, channel: u8, key: u8, vel: u8) {
        self.tx.send(MidiEvent::NoteOn { channel, key, vel }).ok();
    }
    fn note_off(&mut self, channel: u8, key: u8) {
        self.tx.send(MidiEvent::NoteOff { channel, key }).ok();
    }
    fn cc(&mut self, channel: u8, ctrl: u8, value: u8) {
        self.tx
            .send(MidiEvent::ControlChange {
                channel,
                ctrl,
                value,
            })
            .ok();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let mut synth = SynthBackend::new()?;

    let mut input = String::new();
    let midi_in = MidiInput::new("midir reading input")?;

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

    let (_stream, mut synth_conn) = synth.new_output_connection(&"./testdata/sin.sf2");

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            if message.len() == 3 {
                let note = message[1];
                if (21..=108).contains(&note) {
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
