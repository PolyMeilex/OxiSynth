use crate::Synth;

/**
Reverb parameters
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReverbParams {
    pub roomsize: f64,
    pub damp: f64,
    pub width: f64,
    pub level: f64,
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            roomsize: 0.2,
            damp: 0.0,
            width: 0.5,
            level: 0.9,
        }
    }
}

/**
Reverb
 */
impl Synth {
    /**
    Set the parameters for the built-in reverb unit
     */
    pub fn set_reverb_params(&mut self, roomsize: f64, damp: f64, width: f64, level: f64) {
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
    pub fn get_reverb_roomsize(&self) -> f64 {
        self.handle.get_reverb_roomsize()
    }

    /**
    Query the current reverb dumping
     */
    pub fn get_reverb_damp(&self) -> f64 {
        self.handle.get_reverb_damp()
    }

    /**
    Query the current reverb level
     */
    pub fn get_reverb_level(&self) -> f64 {
        self.handle.get_reverb_level()
    }

    /**
    Query the current reverb width
     */
    pub fn get_reverb_width(&self) -> f64 {
        self.handle.get_reverb_width()
    }

    /**
    Query the current reverb params
     */
    pub fn get_reverb(&self) -> ReverbParams {
        ReverbParams {
            roomsize: self.get_reverb_roomsize(),
            damp: self.get_reverb_damp(),
            level: self.get_reverb_level(),
            width: self.get_reverb_width(),
        }
    }
}
