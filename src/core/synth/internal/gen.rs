use crate::core::{
    soundfont::generator::GeneratorType,
    synth::{voice_pool::VoicePool, Channel},
};

/**
Change the value of a generator. This function allows to control
all synthesis parameters in real-time. The changes are additive,
i.e. they add up to the existing parameter value. This function is
similar to sending an NRPN message to the synthesizer. The
function accepts a float as the value of the parameter. The
parameter numbers and ranges are described in the SoundFont 2.01
specification, paragraph 8.1.3, page 48.
 */
pub fn set_gen(channel: &mut Channel, voices: &mut VoicePool, param: GeneratorType, value: f32) {
    channel.set_gen(param, value);
    channel.set_gen_abs(param, 0);

    voices.set_gen(channel.id(), param, value);
}
