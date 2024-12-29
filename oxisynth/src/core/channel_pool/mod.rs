mod channel;
pub(crate) use channel::Channel;

use crate::OxiError;

use super::InterpolationMethod;

pub struct ChannelPool(Vec<Channel>);

impl ChannelPool {
    pub fn new(len: usize, interpolation: InterpolationMethod) -> Self {
        Self(
            (0..len)
                .map(|id| {
                    let mut ch = Channel::new(id);
                    ch.set_interp_method(interpolation);
                    ch
                })
                .collect(),
        )
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
