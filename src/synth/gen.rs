use crate::{engine, Chan, Status, Synth};

type GenParam = engine::gen::GenParam;

/**
Generator interface
 */
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
    pub fn set_gen(&mut self, chan: Chan, param: GenParam, value: f32) -> Status {
        Synth::zero_ok(unsafe { self.handle.set_gen(chan as _, param as _, value) })
    }

    /**
    Retreive the value of a generator. This function returns the value
    set by a previous call 'set_gen()' or by an NRPN message.

    Returns the value of the generator.
     */
    pub fn get_gen(&self, chan: Chan, param: GenParam) -> f32 {
        unsafe { self.handle.get_gen(chan as _, param as _) }
    }
}
