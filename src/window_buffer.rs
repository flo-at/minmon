use crate::{config, Error};

pub struct WindowBuffer<T> {
    buffer: dasp_ring_buffer::Bounded<Vec<T>>,
}

impl<T> TryFrom<&config::FilterWindowConfig> for WindowBuffer<T>
where
    T: Default + Copy,
{
    type Error = Error;

    fn try_from(config: &config::FilterWindowConfig) -> std::result::Result<Self, Self::Error> {
        if config.window_size == 0 {
            return Err(Error(String::from("'window_size' cannot be 0.")));
        }
        let buffer =
            dasp_ring_buffer::Bounded::from(vec![T::default(); config.window_size as usize]);
        Ok(Self { buffer })
    }
}

impl<T> WindowBuffer<T>
where
    T: Copy + Default,
{
    pub fn push(&mut self, data: T) {
        self.buffer.push(data);
    }

    pub fn iter(&self) -> std::iter::Chain<std::slice::Iter<'_, T>, std::slice::Iter<'_, T>> {
        self.buffer.iter()
    }
}
