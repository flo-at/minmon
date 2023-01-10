use crate::window_buffer::WindowBuffer;
use crate::{config, measurement, Error};

pub struct Peak<T> {
    window_buffer: WindowBuffer<Option<T>>,
}

impl<T> TryFrom<&config::FilterPeak> for Peak<T>
where
    T: Default + Copy,
{
    type Error = Error;

    fn try_from(peak: &config::FilterPeak) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            window_buffer: WindowBuffer::try_from(&peak.window_config)?,
        })
    }
}

impl<T> super::Filter<T> for Peak<T>
where
    T: Send + Sync + Copy + measurement::Measurement + std::cmp::Ord,
{
    fn filter(&mut self, data: T) -> T {
        self.window_buffer.push(Some(data));
        self.window_buffer.iter().filter_map(|x| *x).max().unwrap()
    }

    fn error(&mut self) {
        self.window_buffer.push(None);
    }
}
