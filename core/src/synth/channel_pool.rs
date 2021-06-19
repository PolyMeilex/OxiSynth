use std::rc::Rc;

mod channel;
pub use channel::{Channel, InterpolationMethod};

use crate::{soundfont::Preset, OxiError};

pub struct ChannelPool(Vec<Channel>);

impl ChannelPool {
    pub fn new(len: usize, preset: Option<Rc<Preset>>) -> Self {
        let channels = (0..len)
            .map(|id| Channel::new(id, preset.clone()))
            .collect();
        Self(channels)
    }

    pub fn get(&self, id: usize) -> Result<&Channel, OxiError> {
        self.0.get(id).ok_or(OxiError::ChannelOutOfRange)
    }

    pub fn get_mut(&mut self, id: usize) -> Result<&mut Channel, OxiError> {
        self.0.get_mut(id).ok_or(OxiError::ChannelOutOfRange)
    }
}

impl std::ops::Deref for ChannelPool {
    type Target = Vec<Channel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ChannelPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
