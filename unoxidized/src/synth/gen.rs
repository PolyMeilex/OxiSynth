use crate::synth::Synth;
use crate::synth::FLUID_FAILED;
use crate::synth::FLUID_OK;
use crate::synth::GEN_LAST;
use crate::voice::fluid_voice_set_param;

use crate::gen::GenParam;

impl Synth {
    pub unsafe fn set_gen(&mut self, chan: u8, param: GenParam, value: f32) -> i32 {
        let mut i;
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        self.channel[chan as usize].gen[param as usize] = value;
        self.channel[chan as usize].gen_abs[param as usize] = 0 as i32 as i8;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan == chan as u8 {
                fluid_voice_set_param(voice, param as u16, value, 0 as i32);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub fn get_gen(&self, chan: u8, param: GenParam) -> f32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return 0.0f32;
        }
        if (param as u32) >= GEN_LAST {
            log::warn!("Parameter number out of range",);
            return 0.0f32;
        }
        return self.channel[chan as usize].gen[param as usize];
    }
}
