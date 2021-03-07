use crate::synth::Synth;
use crate::synth::GEN_LAST;

use crate::gen::GenParam;

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
                voice.set_param(param, value, 0 as i32);
            }
            i += 1
        }

        Ok(())
    }

    /**
    Retreive the value of a generator. This function returns the value
    set by a previous call 'set_gen()' or by an NRPN message.

    Returns the value of the generator.
     */
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
