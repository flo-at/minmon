use crate::window_buffer::WindowBuffer;
use crate::{config, measurement, Error};

pub struct Average<T> {
    window_buffer: WindowBuffer<Option<T>>,
}

impl<T> TryFrom<&config::FilterAverage> for Average<T>
where
    T: Default + Copy,
{
    type Error = Error;

    fn try_from(average: &config::FilterAverage) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            window_buffer: WindowBuffer::try_from(&average.window_config)?,
        })
    }
}

impl<T> super::Filter<T> for Average<T>
where
    T: Send + Sync + Copy + measurement::Measurement,
    T::Data: std::ops::Add<Output = T::Data>
        + std::ops::Div<Output = T::Data>
        + std::convert::TryFrom<u64>,
    f64: std::convert::From<T::Data>,
    <T::Data as TryFrom<u64>>::Error: std::fmt::Debug,
{
    fn filter(&mut self, data: T) -> T {
        self.window_buffer.push(Some(data));
        let average = self
            .window_buffer
            .iter()
            .filter_map(|x| x.map(|x| f64::from(x.data())))
            .sum::<f64>()
            / <f64 as std::convert::From<u16>>::from(self.window_buffer.len());
        T::new(T::Data::try_from(average.ceil() as u64).unwrap()).unwrap()
    }

    fn error(&mut self) {
        self.window_buffer.push(None);
    }
}
