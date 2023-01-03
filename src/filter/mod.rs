use crate::{config, measurement, Error, Result};

mod average;
mod peak;
mod sum;
use average::Average;
use peak::Peak;
use sum::Sum;

pub trait Filter<T>: Send + Sync
where
    T: measurement::Measurement,
{
    fn filter(&mut self, data: T) -> T;

    fn error(&mut self);
}

pub trait FilterFactory {
    fn factory(filter_config: &config::Filter) -> Result<Box<dyn Filter<Self>>>;
}

macro_rules! make_factory {
    ($T:ty, $( $U:ident ),*) => (
        impl FilterFactory for $T { // TODO rename info fn filter_factory
            fn factory(filter_config: &config::Filter) -> Result<Box<dyn Filter<Self>>> {
                #[allow(unreachable_patterns)]
                match filter_config {
                    $(config::Filter::$U(config) => {
                        Ok(Box::new($U::<$T>::try_from(config)?))
                    },)*
                    _ => Err(Error(String::from("Filter not supported for data type."))),
                }
            }
        }
    )
}
make_factory!(measurement::BinaryState,);
make_factory!(measurement::DataSize, Average, Peak, Sum);
make_factory!(measurement::Level, Average, Peak);
make_factory!(measurement::StatusCode,);
make_factory!(measurement::Temperature, Average, Peak);
