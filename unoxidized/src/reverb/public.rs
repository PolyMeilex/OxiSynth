use super::Reverb;

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

impl Reverb {
    fn set_room_size(&mut self, value: f32) {
        self.roomsize = value * 0.28 + 0.7;
        self.update();
    }

    fn get_room_size(&self) -> f32 {
        return (self.roomsize - 0.7) / 0.28;
    }

    fn set_damp(&mut self, value: f32) {
        self.damp = value * 1.0;
        self.update();
    }

    fn get_damp(&self) -> f32 {
        self.damp / 1.0
    }

    fn set_level(&mut self, value: f32) {
        let value = if value < 0.0 {
            0.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        };
        self.wet = value * 3.0;
        self.update();
    }

    fn get_level(&self) -> f32 {
        return self.wet / 3.0;
    }

    fn set_width(&mut self, value: f32) {
        self.width = value;
        self.update();
    }

    fn get_width(&self) -> f32 {
        self.width
    }
}

impl Reverb {
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
        self.set_room_size(roomsize);
        self.set_damp(damping);
        self.set_width(width);
        self.set_level(level);
    }

    /**
    Turn on/off the built-in reverb unit
     */
    pub fn set_reverb_on(&mut self, on: bool) {
        self.active = on;
    }

    /**
    Query the current reverb room size
     */
    pub fn get_reverb_roomsize(&self) -> f32 {
        self.get_room_size()
    }

    /**
    Query the current reverb dumping
     */
    pub fn get_reverb_damp(&self) -> f32 {
        self.get_damp()
    }

    /**
    Query the current reverb level
     */
    pub fn get_reverb_level(&self) -> f32 {
        self.get_level()
    }

    /**
    Query the current reverb width
     */
    pub fn get_reverb_width(&self) -> f32 {
        self.get_width()
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
