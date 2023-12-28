mod font;
mod midi;
mod params;
mod write;

use crate::core::chorus::Chorus;
use crate::core::font_bank::FontBank;
use crate::core::reverb::Reverb;
pub use crate::core::soundfont::generator::GeneratorType;
pub use crate::core::tuning::{Tuning, TuningManager};
use crate::core::OxiError;
use crate::{MidiEvent, SettingsError, SynthDescriptor};

/**
The synth object

You create a new synthesizer with `Synth::new()`.
Use the settings structure to specify the synthesizer characteristics.

You have to load a SoundFont in order to hear any sound.
For that you use the `Synth::sfload()` function.

You can use the audio driver functions described below to open
the audio device and create a background audio thread.

The API for sending MIDI events is probably what you expect:
`Synth::noteon()`, `Synth::noteoff()`, ...
 */
#[derive(Default)]
pub struct Synth {
    core: crate::core::Synth,
}

impl Synth {
    /**
    Creates a new synthesizer object.

    As soon as the synthesizer is created, it will start playing.
     */
    pub fn new(desc: SynthDescriptor) -> Result<Self, SettingsError> {
        Ok(Synth {
            core: crate::core::Synth::new(desc)?,
        })
    }

    /// Set synth sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.core.settings.sample_rate = sample_rate;
        self.core.voices.set_sample_rate(sample_rate);

        self.core.chorus = Chorus::new(sample_rate, self.core.chorus.active());
    }

    pub fn send_event(&mut self, event: MidiEvent) -> Result<(), OxiError> {
        self.core.send_event(event)
    }

    pub fn font_bank(&self) -> &FontBank {
        &self.core.font_bank
    }

    pub fn font_bank_mut(&mut self) -> &mut FontBank {
        &mut self.core.font_bank
    }
}

// Rverb
impl Synth {
    pub fn get_reverb(&self) -> &Reverb {
        &self.core.reverb
    }

    pub fn get_reverb_mut(&mut self) -> &mut Reverb {
        &mut self.core.reverb
    }
}

// Chorus
impl Synth {
    pub fn chorus(&self) -> &Chorus {
        &self.core.chorus
    }

    pub fn chorus_mut(&mut self) -> &mut Chorus {
        &mut self.core.chorus
    }
}

impl Synth {
    /// Returns the number of MIDI channels that the synthesizer uses internally
    pub fn count_midi_channels(&self) -> usize {
        self.core.channels.len()
    }

    /// Returns the number of effects channels that the synthesizer uses internally
    pub fn count_effects_channels(&self) -> u8 {
        2
    }
}

// Generator interface
impl Synth {
    /**
    Change the value of a generator. This function allows to control
    all synthesis parameters in real-time. The changes are additive,
    i.e. they add up to the existing parameter value. This function is
    similar to sending an NRPN message to the synthesizer. The
    function accepts a float as the value of the parameter. The
    parameter numbers and ranges are described in the SoundFont 2.01
    specification, paragraph 8.1.3, page 48.
     */
    pub fn set_gen(
        &mut self,
        chan: usize,
        param: GeneratorType,
        value: f32,
    ) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(chan)?;

        crate::core::synth::midi::set_gen(channel, &mut self.core.voices, param, value);

        Ok(())
    }

    /// Retreive the value of a generator. This function returns the value
    /// set by a previous call 'set_gen()' or by an NRPN message.
    ///
    /// Returns the value of the generator.
    pub fn gen(&self, chan: u8, param: GeneratorType) -> Result<f32, OxiError> {
        let channel = self.core.channels.get(chan as usize)?;
        Ok(channel.gen(param))
    }
}

// Tuning
impl Synth {
    /// Select a tuning for a channel.
    pub fn channel_set_tuning(&mut self, chan: u8, tuning: Tuning) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(chan as usize)?;
        channel.set_tuning(Some(tuning));
        Ok(())
    }

    /// Set the tuning to the default well-tempered tuning on a channel.
    pub fn channel_reset_tuning(&mut self, chan: u8) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(chan as usize)?;
        channel.set_tuning(None);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{MidiEvent, SoundFont, Synth, SynthDescriptor};
    use std::{fs::File, io::Write, slice::from_raw_parts};

    #[test]
    fn synth_sf2() {
        let mut pcm = File::create("Boomwhacker.sf2.pcm").unwrap();

        let mut synth = Synth::new(SynthDescriptor::default()).unwrap();

        let mut file = std::fs::File::open("./testdata/Boomwhacker.sf2").unwrap();
        let font = SoundFont::load(&mut file).unwrap();

        synth.add_font(font, true);

        let mut samples = [0f32; 44100 * 2];

        synth
            .send_event(MidiEvent::NoteOn {
                channel: 0,
                key: 60,
                vel: 127,
            })
            .unwrap();

        synth.write(samples.as_mut());
        pcm.write_all(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();

        synth
            .send_event(crate::core::MidiEvent::NoteOff {
                channel: 0,
                key: 60,
            })
            .unwrap();

        synth.write(samples.as_mut());
        pcm.write_all(unsafe {
            from_raw_parts(samples.as_ptr() as _, std::mem::size_of_val(&samples))
        })
        .unwrap();

        drop(synth);
    }
}
