use crate::{Error, Result};

pub trait Measurement: std::fmt::Display + Copy + Clone + Default {
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

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Default)]
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

#[derive(PartialEq, Eq, Copy, Clone, Default)]
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

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Default)]
pub struct Temperature {
    data: i16,
}

impl_Display!(Temperature);

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
