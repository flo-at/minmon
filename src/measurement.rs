use crate::{Error, Result};

pub trait Measurement: std::fmt::Display + std::fmt::Debug + Copy + Clone + Default {
    type Data: Copy + Clone + Default;
    const UNIT: &'static str;

    fn new(data: Self::Data) -> Result<Self>
    where
        Self: Sized;

    fn data(&self) -> Self::Data;
}

macro_rules! impl_Display {
    ($T:ty) => {
        impl std::fmt::Display for $T {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}", self.data, Self::UNIT)
            }
        }
    };
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct BinaryState {
    data: bool,
}

impl_Display!(BinaryState);

impl Measurement for BinaryState {
    type Data = bool;
    const UNIT: &'static str = "";

    fn new(data: Self::Data) -> Result<Self> {
        Ok(Self { data })
    }

    fn data(&self) -> Self::Data {
        self.data
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Default, Debug)]
pub struct DataSize {
    data: u64,
}

impl DataSize {
    pub const MAX: Self = Self { data: u64::MAX };
    const MAX_PRETTY_PRECISION: usize = 3;

    pub fn as_string_binary(&self) -> String {
        let mut bytes = self.data as f64;
        let mut unit = Self::UNIT;
        for test_unit in ["KiB", "MiB", "GiB"] {
            if bytes >= 1024. {
                bytes /= 1024.;
                unit = test_unit;
            } else {
                break;
            }
        }
        let num_string = format!("{bytes:.*}", Self::MAX_PRETTY_PRECISION);
        let num_trim_string = num_string.trim_end_matches('0');
        let num_trim_string = num_trim_string.trim_end_matches('.');
        format!("{num_trim_string}{unit}")
    }

    pub fn as_string_decimal(&self) -> String {
        let mut bytes = self.data as f64;
        let mut unit = Self::UNIT;
        for test_unit in ["kB", "MB", "GB"] {
            if bytes >= 1000. {
                bytes /= 1000.;
                unit = test_unit;
            } else {
                break;
            }
        }
        let num_string = format!("{bytes:.*}", Self::MAX_PRETTY_PRECISION);
        let num_trim_string = num_string.trim_end_matches('0');
        let num_trim_string = num_trim_string.trim_end_matches('.');
        format!("{num_trim_string}{unit}")
    }
}

impl_Display!(DataSize);

impl Measurement for DataSize {
    type Data = u64;
    const UNIT: &'static str = "B";

    fn new(data: Self::Data) -> Result<Self> {
        Ok(Self { data })
    }

    fn data(&self) -> Self::Data {
        self.data
    }
}

impl std::ops::Add for DataSize {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            data: self.data + other.data,
        }
    }
}

impl std::iter::Sum<DataSize> for DataSize {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = DataSize>,
    {
        iter.fold(Self::default(), |a, b| a + b)
    }
}

impl std::ops::Sub for DataSize {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            data: self.data - other.data,
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug, Default)]
pub struct Level {
    data: u8,
}

impl_Display!(Level);

impl Measurement for Level {
    type Data = u8;
    const UNIT: &'static str = "%";

    fn new(data: Self::Data) -> Result<Self> {
        if data > 100 {
            Err(Error(String::from("'level' cannot be greater than 100.")))
        } else {
            Ok(Self { data })
        }
    }

    fn data(&self) -> Self::Data {
        self.data
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct StatusCode {
    data: u8,
}

impl_Display!(StatusCode);

impl Measurement for StatusCode {
    type Data = u8;
    const UNIT: &'static str = "";

    fn new(data: Self::Data) -> Result<Self> {
        Ok(Self { data })
    }

    fn data(&self) -> Self::Data {
        self.data
    }
}

#[cfg(feature = "sensors")]
#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug, Default)]
pub struct Temperature {
    data: i16,
}

#[cfg(feature = "sensors")]
impl_Display!(Temperature);

#[cfg(feature = "sensors")]
impl Measurement for Temperature {
    type Data = i16;
    const UNIT: &'static str = "°C";

    fn new(data: Self::Data) -> Result<Self> {
        if data < -273 {
            Err(Error(String::from(
                "'temperature' cannot be less than -273°C.",
            )))
        } else {
            Ok(Self { data })
        }
    }

    fn data(&self) -> Self::Data {
        self.data
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Default, Debug)]
pub struct Integer {
    data: i64,
}

impl_Display!(Integer);

impl Measurement for Integer {
    type Data = i64;
    const UNIT: &'static str = "";

    fn new(data: Self::Data) -> Result<Self> {
        Ok(Self { data })
    }

    fn data(&self) -> Self::Data {
        self.data
    }
}

impl std::ops::Add for Integer {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            data: self.data + other.data,
        }
    }
}

impl std::iter::Sum<Integer> for Integer {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Integer>,
    {
        iter.fold(Self::default(), |a, b| a + b)
    }
}

impl std::ops::Sub for Integer {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            data: self.data - other.data,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_data_size_as_string_binary() {
        let data_size = DataSize::new(999).unwrap();
        assert_eq!(format!("{}", data_size.as_string_binary()), "999B");
        let data_size = DataSize::new(9999).unwrap();
        assert_eq!(format!("{}", data_size.as_string_binary()), "9.765KiB");
        let data_size = DataSize::new(9999999).unwrap();
        assert_eq!(format!("{}", data_size.as_string_binary()), "9.537MiB");
        let data_size = DataSize::new(1024).unwrap();
        assert_eq!(format!("{}", data_size.as_string_binary()), "1KiB");
        let data_size = DataSize::new(1024 * 1024).unwrap();
        assert_eq!(format!("{}", data_size.as_string_binary()), "1MiB");
        let data_size = DataSize::new(1024 * 1024 * 1024).unwrap();
        assert_eq!(format!("{}", data_size.as_string_binary()), "1GiB");
    }
}
