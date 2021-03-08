use crate::Synth;

use crate::oxi::synth::reverb::ReverbParams;

/**
Reverb
 */
impl Synth {
    /**
    Set the parameters for the built-in reverb unit
     */
    pub fn set_reverb_params(&mut self, roomsize: f32, damp: f32, width: f32, level: f32) {
        self.handle.set_reverb_params(roomsize, damp, width, level);
    }

    /**
    Set the parameters for the built-in reverb unit
     */
    pub fn set_reverb(&mut self, params: &ReverbParams) {
        self.set_reverb_params(params.roomsize, params.damp, params.width, params.level);
    }

    /**
    Turn on/off the built-in reverb unit
     */
    pub fn set_reverb_on(&mut self, on: bool) {
        self.handle.set_reverb_on(on as _);
    }

    /**
    Query the current reverb room size
     */
    pub fn get_reverb_roomsize(&self) -> f32 {
        self.handle.get_reverb_roomsize()
    }

    /**
    Query the current reverb dumping
     */
    pub fn get_reverb_damp(&self) -> f32 {
        self.handle.get_reverb_damp()
    }

    /**
    Query the current reverb level
     */
    pub fn get_reverb_level(&self) -> f32 {
        self.handle.get_reverb_level()
    }

    /**
    Query the current reverb width
     */
    pub fn get_reverb_width(&self) -> f32 {
        self.handle.get_reverb_width()
    }

    /**
    Query the current reverb params
     */
    pub fn get_reverb(&self) -> ReverbParams {
        self.handle.get_reverb()
    }
}
