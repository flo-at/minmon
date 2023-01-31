use super::DataSource;
use crate::measurement::Measurement;
use crate::{config, measurement};
use crate::{Error, Result};
use async_trait::async_trait;

type Item = measurement::Temperature;

pub struct Temperature {
    id: Vec<String>,
    sensors: Vec<SensorsId>,
}

struct SensorsId {
    pub sensor: String,
    pub label: String,
}

impl std::fmt::Display for SensorsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.sensor, self.label)
    }
}

impl Temperature {
    fn get_temperature(sensors_id: &SensorsId) -> Result<Item> {
        let sensor = sensors::Sensors::new();
        // these unwraps cannot happen here as they are checked in try_from
        for chip in sensor.detected_chips(&sensors_id.sensor).unwrap() {
            if let Some(feature) = chip
                .into_iter()
                .find(|x| x.get_label().unwrap() == sensors_id.label)
            {
                if let Some(subfeature) = feature.into_iter().find(|x| {
                    *x.subfeature_type() == sensors::SubfeatureType::SENSORS_SUBFEATURE_TEMP_INPUT
                }) {
                    return Item::new(
                        subfeature
                            .get_value()
                            .map(|x| x as <Item as Measurement>::Data)
                            .map_err(|x| Error(format!("Could not read temperature: {x}")))?,
                    );
                };
            }
        }
        Err(Error(String::from("Could not read temperature.")))
    }
}

impl TryFrom<&config::SensorsId> for SensorsId {
    type Error = Error;

    fn try_from(sensors_id: &config::SensorsId) -> std::result::Result<Self, Self::Error> {
        let mut res = None;
        let sensor = sensors::Sensors::new();
        for chip in sensor.detected_chips(sensors_id.sensor()).map_err(|x| {
            Error(format!(
                "Failed to parse sensor name '{}': {x}",
                sensors_id.sensor()
            ))
        })? {
            // these unwraps cannot happen when using the iterator
            let chip_name = chip.get_name().unwrap();
            for feature in chip
                .into_iter()
                .filter(|x| *x.feature_type() == sensors::FeatureType::SENSORS_FEATURE_TEMP)
            {
                let label = feature.get_label().unwrap();
                if let Some(feature_label) = &sensors_id.label() {
                    if *feature_label != label {
                        continue;
                    }
                }
                for _ in feature.into_iter().filter(|x| {
                    *x.subfeature_type() == sensors::SubfeatureType::SENSORS_SUBFEATURE_TEMP_INPUT
                }) {
                    if res.is_some() {
                        return Err(Error(format!("Sensor '{sensors_id}' is not unique.")));
                    }
                    res = Some(Self {
                        sensor: chip_name.clone(),
                        label: label.clone(),
                    });
                }
            }
        }
        res.ok_or_else(|| Error(format!("Sensor '{sensors_id}' not found.")))
    }
}

impl TryFrom<&config::Check> for Temperature {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
        if let config::CheckType::Temperature(temperature) = &check.type_ {
            let sensors = temperature
                .sensors
                .iter()
                .map(SensorsId::try_from)
                .collect::<Result<Vec<SensorsId>>>()?;
            let id = sensors.iter().map(|x| x.to_string()).collect();
            Ok(Self { id, sensors })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for Temperature {
    type Item = measurement::Temperature;

    async fn get_data(&mut self) -> Result<Vec<Result<Self::Item>>> {
        Ok(self
            .sensors
            .iter()
            .map(Temperature::get_temperature)
            .collect())
    }

    fn format_data(data: &Self::Item) -> String {
        format!("temperature {data}")
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}
