use crate::filter::Filter;
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

impl<T> Filter<T> for Average<T>
where
    T: Send + Sync + Copy + measurement::Measurement,
    T::Data: std::ops::Add<Output = T::Data>
        + std::ops::Div<Output = T::Data>
        + std::convert::TryFrom<num_bigint::BigInt>,
    num_bigint::BigInt: std::convert::From<T::Data>,
    <T::Data as TryFrom<num_bigint::BigInt>>::Error: std::fmt::Debug,
{
    fn filter(&mut self, data: T) -> T {
        use num_integer::Integer;
        self.window_buffer.push(Some(data));
        let num_values = self.window_buffer.iter().filter(|x| x.is_some()).count();
        let (quotient, remainder) = self
            .window_buffer
            .iter()
            .filter_map(|x| x.map(|x| num_bigint::BigInt::from(x.data())))
            .sum::<num_bigint::BigInt>()
            .div_rem(&num_values.into());
        let average = if remainder >= ((num_values + 1) / 2).into() {
            quotient + 1
        } else {
            quotient
        };
        T::new(T::Data::try_from(average).unwrap()).unwrap()
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
    async fn test_average_filter_rounding() {
        let mut filter = Average::<measurement::Level>::try_from(&config::FilterAverage {
            window_config: config::FilterWindowConfig { window_size: 5 },
        })
        .unwrap();
        assert_eq!(filter.filter(level(1)), level(1)); // identity
        assert_eq!(filter.filter(level(10)), level(6)); // even window, round up
        assert_eq!(filter.filter(level(30)), level(14)); // odd window, round up
        assert_eq!(filter.filter(level(4)), level(11)); // even window, round down
        assert_eq!(filter.filter(level(3)), level(10)); // odd window, round down
    }

    #[tokio::test]
    async fn test_average_filter_errors() {
        let mut filter = Average::<measurement::Level>::try_from(&config::FilterAverage {
            window_config: config::FilterWindowConfig { window_size: 9 },
        })
        .unwrap();
        filter.error();
        assert_eq!(filter.filter(level(1)), level(1));
        filter.error();
        assert_eq!(filter.filter(level(3)), level(2));
        filter.error();
        assert_eq!(filter.filter(level(5)), level(3));
        filter.error();
        assert_eq!(filter.filter(level(7)), level(4));
        filter.error();
        assert_eq!(filter.filter(level(9)), level(5));
    }

    #[tokio::test]
    async fn test_average_filter_window() {
        let mut filter = Average::<measurement::Level>::try_from(&config::FilterAverage {
            window_config: config::FilterWindowConfig { window_size: 3 },
        })
        .unwrap();
        assert_eq!(filter.filter(level(1)), level(1));
        filter.error();
        assert_eq!(filter.filter(level(2)), level(2));
        assert_eq!(filter.filter(level(3)), level(3)); // rolls of 1
        assert_eq!(filter.filter(level(4)), level(3));
        filter.error(); // rolls of 2
        assert_eq!(filter.filter(level(5)), level(5)); // tolls of 3
    }
}
