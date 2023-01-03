use crate::filter::Filter;
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

impl<T> Filter<T> for Peak<T>
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

#[cfg(test)]
mod test {
    use super::*;

    fn level(level: u8) -> measurement::Level {
        use measurement::Measurement;
        measurement::Level::new(level).unwrap()
    }

    #[tokio::test]
    async fn test_peak_filter() {
        let mut filter = Peak::<measurement::Level>::try_from(&config::FilterPeak {
            window_config: config::FilterWindowConfig { window_size: 3 },
        })
        .unwrap();
        assert_eq!(filter.filter(level(1)), level(1));
        assert_eq!(filter.filter(level(2)), level(2));
        assert_eq!(filter.filter(level(3)), level(3));
        assert_eq!(filter.filter(level(2)), level(3)); // rolls of 1
        assert_eq!(filter.filter(level(1)), level(3)); // rolls of 2
        assert_eq!(filter.filter(level(2)), level(2)); // rolls of 3
        filter.error(); // rolls of 2
        assert_eq!(filter.filter(level(10)), level(10)); // rolls of 1
        filter.error(); // rolls of 2
        assert_eq!(filter.filter(level(9)), level(10));
        assert_eq!(filter.filter(level(8)), level(9)); // rolls of 10
        filter.error();
        filter.error(); // rolls of 9
        assert_eq!(filter.filter(level(1)), level(1)); // rolls of 8
    }
}
