use crate::{generator::GenParam, synth::Synth};

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
        param: GenParam,
        value: f32,
    ) -> Result<(), &'static str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            channel.set_gen(param as usize, value);
            channel.set_gen_abs(param as usize, 0);

            self.voices.set_gen(chan, param, value);

            Ok(())
        } else {
            log::error!("Channel out of range");
            Err("Channel out of range")
        }
    }

    /**
    Retreive the value of a generator. This function returns the value
    set by a previous call 'set_gen()' or by an NRPN message.

    Returns the value of the generator.
     */
    pub fn get_gen(&self, chan: u8, param: GenParam) -> Result<f32, &'static str> {
        if let Some(channel) = self.channels.get(chan as usize) {
            Ok(channel.gen(param as usize))
        } else {
            log::error!("Channel out of range");
            Err("Channel out of range")
        }
    }
}
