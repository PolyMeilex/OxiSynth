use crate::synth::Synth;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReverbParams {
    pub roomsize: f32,
    pub damp: f32,
    pub width: f32,
    pub level: f32,
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

impl Synth {
    /**
    Set the parameters for the built-in reverb unit
     */
    pub fn set_reverb(&mut self, params: &ReverbParams) {
        self.set_reverb_params(params.roomsize, params.damp, params.width, params.level);
    }

    /**
    Set the parameters for the built-in reverb unit
     */
    pub fn set_reverb_params(&mut self, roomsize: f32, damping: f32, width: f32, level: f32) {
        self.reverb.set_room_size(roomsize);
        self.reverb.set_damp(damping);
        self.reverb.set_width(width);
        self.reverb.set_level(level);
    }

    /**
    Turn on/off the built-in reverb unit
     */
    pub fn set_reverb_on(&mut self, on: bool) {
        self.settings.reverb_active = on;
    }

    /**
    Query the current reverb room size
     */
    pub fn get_reverb_roomsize(&self) -> f32 {
        self.reverb.get_room_size()
    }

    /**
    Query the current reverb dumping
     */
    pub fn get_reverb_damp(&self) -> f32 {
        self.reverb.get_damp()
    }

    /**
    Query the current reverb level
     */
    pub fn get_reverb_level(&self) -> f32 {
        self.reverb.get_level()
    }

    /**
    Query the current reverb width
     */
    pub fn get_reverb_width(&self) -> f32 {
        self.reverb.get_width()
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
