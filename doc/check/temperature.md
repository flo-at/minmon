# Temperature
Checks a temperature using lm_sensors.\
This check is only available if MinMon is built with the `sensors` feature.

## Check options
Wildcards are allowed in the sensor name as long as only one sensor is matched.\
Specifying the label is optional as long as there is only one temperature feature in the sensor.

| name | example | optional | default |
|:---|:---|:---:|:---|
| sensors | `["acpitz-*", {sensor = "coretemp-*", label = "Core 0"}]` | ❌ | |

### sensors
List of sensors to check.
Each entry can be either a string that matches exactly one sensor or an object that matches a specific sensor feature by the sensor's name and its feature's label.

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| temperature | `80` | ❌ | |

### temperature
Temperature threshold in °C.
The alarm will be triggered if the measured value exceeds this value.

## IDs
Names of the sensors and labels as provided by lm_sensors (e.g. `acpitz-acpi-0[temp1]).

## Placeholders
- `temperature`: Measured temperature (in °C).
