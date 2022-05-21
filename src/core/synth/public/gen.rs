use crate::core::{soundfont::generator::GeneratorType, synth::internal, OxiError, Synth};

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
        let channel = self.channels.get_mut(chan as usize)?;

        internal::set_gen(channel, &mut self.voices, param, value);
        Ok(())
    }

    /**
    Retreive the value of a generator. This function returns the value
    set by a previous call 'set_gen()' or by an NRPN message.

    Returns the value of the generator.
     */
    pub fn gen(&self, chan: u8, param: GeneratorType) -> Result<f32, OxiError> {
        let channel = self.channels.get(chan as usize)?;
        Ok(internal::gen(channel, param))
    }
}
