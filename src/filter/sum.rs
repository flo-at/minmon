use crate::filter::Filter;
use crate::window_buffer::WindowBuffer;
use crate::{config, measurement, Error};

pub struct Sum<T> {
    window_buffer: WindowBuffer<Option<T>>,
}

impl<T> TryFrom<&config::FilterSum> for Sum<T>
where
    T: Default + Copy,
{
    type Error = Error;

    fn try_from(sum: &config::FilterSum) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            window_buffer: WindowBuffer::try_from(&sum.window_config)?,
        })
    }
}

impl<T> Filter<T> for Sum<T>
where
    T: Send + Sync + Copy + measurement::Measurement,
    T: std::ops::Add<Output = T> + std::iter::Sum,
{
    fn filter(&mut self, data: T) -> T {
        self.window_buffer.push(Some(data));
        self.window_buffer.iter().filter_map(|x| *x).sum()
    }

    fn error(&mut self) {
        self.window_buffer.push(None);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn size(size: u64) -> measurement::DataSize {
        use measurement::Measurement;
        measurement::DataSize::new(size).unwrap()
    }

    #[tokio::test]
    async fn test_sum_filter() {
        let mut filter = Sum::<measurement::DataSize>::try_from(&config::FilterSum {
            window_config: config::FilterWindowConfig { window_size: 3 },
        })
        .unwrap();
        assert_eq!(filter.filter(size(1)), size(1)); // identity
        assert_eq!(filter.filter(size(2)), size(3));
        assert_eq!(filter.filter(size(3)), size(6));
        assert_eq!(filter.filter(size(4)), size(9)); // rolls of 1
        filter.error(); // rolls of 2
        assert_eq!(filter.filter(size(5)), size(9)); // rolls of 3
        filter.error(); // rolls of 4
        assert_eq!(filter.filter(size(0)), size(5));
        assert_eq!(filter.filter(size(0)), size(0)); // rolls of 5
    }
}
