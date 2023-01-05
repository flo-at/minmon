# Check
Besides the generic options, each check has alarms attached to it.
Some of the type-specific options go into the check's section, some into the sections of the alarms - as listed below.
A single check can generate data for one or more "IDs", e.g. mountpoints. Each alarm is instantiated for every ID.

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| interval | `60` | ✔ | `300` |
| name | `Foobar` | ❌ | |
| timeout | `1` | ✔ | `5` |
| placeholders | `{"internal_check_id" = "id_foobar"}` | ✔ | |
| type | `FilesystemUsage` | ❌ | |
| alarms | List of [Alarm](#alarm) | ✔ | |

# FilesystemUsage
Reads the filesystem usage of the given mountpoints.
This check reads the "available blocks" (not "free blocks") i.e. blocks available to unprivileged users.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| mountpoints | `["/srv", "/home"]` | ❌ | | |

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | |

## IDs
Equivalent to the "mountpoints" config option.

## Placeholders
- `level`: Filesystem space usage (in percent).

# MemoryUsage
Reads the system memory (physical RAM) and swap file usage.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| memory | `true` | ✔ | `false` | |
| swap | `true` | ✔ | `false` | |

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | |

## IDs
- `Memory`
- `Swap`

## Placeholders
- `level`: Memory space usage (in percent).

# PressureAverage
Reads the average values from the Linux kernel's Pressure Stall Information (PSI).
See the [kernel documentation](https://www.kernel.org/doc/html/latest/accounting/psi.html) for more information.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| cpu | `true` | ✔ | `false` | |
| io | `None`, `Some`, `Full`, `Both` | ✔ | `None` | |
| memory | `None`, `Some`, `Full`, `Both` | ✔ | `None` | |
| avg10 | `true` | ✔ | `false` | |
| avg60 | `true` | ✔ | `false` | |
| avg300 | `true` | ✔ | `false` | |

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | | |

## IDs
- All combinations of `cpu/{avg10,avg60,avg300}`
- All combinations of `{io,memory}/{some,full}/{avg10,avg60,avg300}`

## Placeholders
- `level`: Pressure average (in percent).

# ProcessExitStatus
Runs a process and checks its exit status code.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| path | `/usr/bin/echo` | ❌ | |
| arguments | `["-e", "Checking things.."]` | ✔ | |
| environment_variables | `{"FOO": "BAR"}` | ✔ | |
| working_directory | `/home/user/` | ✔ | inherited (\*) |
| uid | `1000` | ✔ | inherited (*) |
| gid | `1000` | ✔ | inherited (*) |

(\*) Inherited from MinMon's process.

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| status_codes | `[1, 255]` | ✔ | `[0]` |

## IDs
Name of the file given by the path.

## Placeholders
- `status_code`: Process exit status code.

# SystemdUnitStatus
Checks whether a systemd unit is active or not.\
The `uid` value is optional. If it is non-zero, `systemctl --user` will be run with the given UID.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| units | `[{unit = "foo.service", uid = 1000}]` | ❌ | |

## Alarm options
None.

## IDs
Unit names with UIDs (if non-zero) (e.g. `foo.service[1000]`).

## Placeholders
- `state`: `true` if service is active else `false`.

# Temperature
Checks a temperature using lm_sensors.

## Check options
Wildcards are allowed in the sensor name as long as only one sensor is matched.\
Specifying the feature is optional as long as there is only one temperature feature in the sensor.

| name | example | optional | default |
|:---|:---|:---:|:---|
| sensors | `[{sensor = "acpitz-*", feature = "temp1"}]` | ❌ | |

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| temperature | `80` | ❌ | |

## IDs
Names of the sensors and features as provided by lm_sensors (e.g. `acpitz-acpi-0/temp1).

## Placeholders
- `temperature`: Measured temperature (in °C).

---

# Alarm

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `Foobar` | ❌ | |
| action | `FooAction` | ❌ | |
| placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| cycles | `3` | ❌ | `1` |
| repeat_cycles | `100` | ✔ | |
| recover_action | `FooAction` | ✔ | |
| recover_placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| recover_cycles | `3` | ✔ | `1` |
| error_action | `FooAction` | ✔ | |
| error_placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| error_repeat_cycles | `100` | ✔ | |
| invert | `true` | ✔ | `false` |
