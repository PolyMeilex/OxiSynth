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
    /// Turn on/off the built-in Reverb unit
    pub fn set_active(&mut self, on: bool) {
        self.active = on;
    }

    /// Check if Reverb is on/off
    pub fn get_active(&self) -> bool {
        self.active
    }

    /// Set the current reverb room size
    fn set_room_size(&mut self, value: f32) {
        self.roomsize = value * 0.28 + 0.7;
    }

    /// Query the current reverb room size
    pub fn get_room_size(&self) -> f32 {
        (self.roomsize - 0.7) / 0.28
    }

    /// Set the current reverb dumping
    fn set_damp(&mut self, value: f32) {
        self.damp = value * 1.0;
    }

    /// Query the current reverb dumping
    pub fn get_damp(&self) -> f32 {
        self.damp / 1.0
    }

    /// Set the current reverb level
    fn set_level(&mut self, value: f32) {
        let value = if value < 0.0 {
            0.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        };
        self.wet = value * 3.0;
    }

    /// Query the current reverb level
    pub fn get_level(&self) -> f32 {
        self.wet / 3.0
    }

    /// Set the current reverb width
    fn set_width(&mut self, value: f32) {
        self.width = value;
    }

    /// Query the current reverb width
    pub fn get_width(&self) -> f32 {
        self.width
    }
}

impl Reverb {
    /// Set the parameters for the built-in reverb unit
    pub fn set_reverb(&mut self, params: &ReverbParams) {
        self.set_reverb_params(params.roomsize, params.damp, params.width, params.level);
    }

    /// Set the parameters for the built-in reverb unit
    pub fn set_reverb_params(&mut self, roomsize: f32, damping: f32, width: f32, level: f32) {
        self.set_room_size(roomsize);
        self.set_damp(damping);
        self.set_width(width);
        self.set_level(level);
        self.update();
    }

    /// Query the current reverb params
    pub fn get_reverb(&self) -> ReverbParams {
        ReverbParams {
            roomsize: self.get_room_size(),
            damp: self.get_damp(),
            level: self.get_level(),
            width: self.get_width(),
        }
    }
}
