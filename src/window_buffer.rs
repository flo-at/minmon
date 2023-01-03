use crate::{config, Error};

pub struct WindowBuffer<T> {
    buffer: dasp_ring_buffer::Bounded<Vec<T>>,
    fill_mode: config::WindowFillMode,
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
        Ok(Self {
            buffer,
            fill_mode: config.window_fill_mode,
        })
    }
}

impl<T> WindowBuffer<T>
where
    T: Copy + Default,
{
    pub fn push(&mut self, data: T) {
        'outer: {
            if self.buffer.len() < self.buffer.max_len() - 1 {
                let fill_value = match self.fill_mode {
                    config::WindowFillMode::Grow => break 'outer,
                    config::WindowFillMode::Fill => data,
                };
                while self.buffer.len() < self.buffer.max_len() - 1 {
                    self.buffer.push(fill_value);
                }
            }
        }
        self.buffer.push(data);
    }

    pub fn len(&self) -> u16 {
        self.buffer.len() as u16
    }

    pub fn iter(&self) -> std::iter::Chain<std::slice::Iter<T>, std::slice::Iter<T>> {
        self.buffer.iter()
    }
}
