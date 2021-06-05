use std::rc::Rc;

mod channel;
pub use channel::{Channel, InterpolationMethod};

use crate::soundfont::Preset;

pub struct ChannelPool(Vec<Channel>);

impl ChannelPool {
    pub fn new(len: usize, preset: Option<Rc<Preset>>) -> Self {
        let channels = (0..len)
            .map(|id| Channel::new(id, preset.clone()))
            .collect();
        Self(channels)
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
