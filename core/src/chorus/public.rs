use super::Chorus;

/**
Chorus type
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ChorusMode {
    Sine = 0,
    Triangle = 1,
}

impl Default for ChorusMode {
    fn default() -> Self {
        ChorusMode::Sine
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChorusParams {
    pub nr: u32,
    pub level: f32,
    /// Speed in Hz
    pub speed: f32,
    /// Depth in mS
    pub depth: f32,
    /// Mode
    pub mode: ChorusMode,
}

impl Default for ChorusParams {
    fn default() -> Self {
        Self {
            nr: 3,
            level: 2.0,
            speed: 0.3,
            depth: 8.0,
            mode: ChorusMode::default(),
        }
    }
}

impl Chorus {
    /// Turn on/off the built-in chorus unit
    pub fn set_active(&mut self, on: bool) {
        self.active = on;
    }

    /// Check if Chorus is on/off
    pub fn get_active(&self) -> bool {
        self.active
    }

    /// Set the current chorus nr
    fn set_nr(&mut self, nr: u32) {
        self.new_number_blocks = nr;
    }

    /// Query the current chorus nr
    pub fn get_nr(&self) -> u32 {
        self.number_blocks
    }

    /// Set the current chorus level
    fn set_level(&mut self, level: f32) {
        self.new_level = level;
    }

    /// Query the current chorus level
    pub fn get_level(&self) -> f32 {
        self.level
    }

    /// Set the current chorus speed (Hz)
    fn set_speed_hz(&mut self, speed_hz: f32) {
        self.new_speed_hz = speed_hz;
    }

    /// Query the current chorus speed (Hz)
    pub fn get_speed_hz(&self) -> f32 {
        self.speed_hz
    }

    /// Set the current chorus depth (mS)
    fn set_depth_ms(&mut self, depth_ms: f32) {
        self.new_depth_ms = depth_ms;
    }

    /// Query the current chorus depth (mS)
    pub fn get_depth_ms(&self) -> f32 {
        self.depth_ms
    }

    /// Set the current chorus mode
    fn set_mode(&mut self, mode: ChorusMode) {
        self.new_type = mode;
    }

    /// Query the current chorus mode
    pub fn get_mode(&self) -> ChorusMode {
        self.type_0
    }
}

impl Chorus {
    /**
    Set up the chorus. It should be turned on with Chorus::set_active().
    If faulty parameters are given, all new settings are discarded.
    Keep in mind, that the needed CPU time is proportional to `nr`.
     */
    pub fn set_chorus(&mut self, params: &ChorusParams) {
        self.set_chorus_params(
            params.nr,
            params.level,
            params.speed,
            params.depth,
            params.mode,
        );
    }

    /**
    Set up the chorus. It should be turned on with Chorus::set_active().
    If faulty parameters are given, all new settings are discarded.
    Keep in mind, that the needed CPU time is proportional to `nr`.
     */
    pub fn set_chorus_params(
        &mut self,
        nr: u32,
        level: f32,
        speed: f32,
        depth_ms: f32,
        type_0: ChorusMode,
    ) {
        self.set_nr(nr);
        self.set_level(level);
        self.set_speed_hz(speed);
        self.set_depth_ms(depth_ms);
        self.set_mode(type_0);
        self.update();
    }

    /**
    Query the current chorus params
     */
    pub fn get_chorus(&self) -> ChorusParams {
        ChorusParams {
            nr: self.get_nr(),
            level: self.get_level(),
            speed: self.get_speed_hz(),
            depth: self.get_depth_ms(),
            mode: self.get_mode(),
        }
    }
}
