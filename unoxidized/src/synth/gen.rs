#![forbid(unsafe_code)]

use crate::synth::Synth;
use crate::synth::GEN_LAST;

use crate::gen::GenParam;

impl Synth {
    pub fn set_gen(&mut self, chan: u8, param: GenParam, value: f32) -> Result<(), ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range");
            return Err(());
        }
        self.channel[chan as usize].gen[param as usize] = value;
        self.channel[chan as usize].gen_abs[param as usize] = 0 as i32 as i8;
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan == chan as u8 {
                voice.set_param(param as u16, value, 0 as i32);
            }
            i += 1
        }

        Ok(())
    }

    pub fn get_gen(&self, chan: u8, param: GenParam) -> f32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range");
            0.0
        } else if (param as u8) >= GEN_LAST {
            log::warn!("Parameter number out of range");
            0.0
        } else {
            self.channel[chan as usize].gen[param as usize]
        }
    }
}
