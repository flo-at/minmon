# Check
Besides the generic options listed below, checks have additional options that are specific to their type.
A single check can generate data for one or more "IDs", e.g. mountpoints, temperature sensors, and so on.
Each alarm is instantiated for every ID.

## Generic options (for all check types)
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| interval | `60` | ✔ | `300` |
| name | `"Foobar"` | ❌ | |
| timeout | `1` | ✔ | min(`5`, interval) |
| placeholders | `{"internal_check_id" = "id_foobar"}` | ✔ | |
| type | `"FilesystemUsage"` | ❌ | |
| alarms | see below | ✔ | |

### disable
If `true`, the check is disabled and will not be instantiated.

### interval
The time between two consecutive checks in seconds.
Has to be greater or equal to the timeout.

### name
The name of the check. It is used for logging and the `check_name` placeholder.
Must be unique.

### timeout
The maximum time in seconds a check may take to return its measurement data before being interrupted.
Has to be less or equal to the interval.

### placeholders
Custom placeholders that will be merged with the ones of the alarms/actions.

### type
Type of the check as listed below.
This determines which specific check and alarm options are available.

One of:
- [DockerContainerStatus](./check/docker_container_status.md)
- [FilesystemUsage](./check/filesystem_usage.md)
- [MemoryUsage](./check/memory_usage.md)
- [NetworkThroughput](./check/network_throughput.md)
- [PressureAverage](./check/pressure_average.md)
- [ProcessExitStatus](./check/process_exit_status.md)
- [SystemdUnitStatus](./check/systemd_unit_status.md)
- [Temperature](./check/temperature.md)

### alarm
List of [alarms](#alarm).

## Generic placeholders (for all check types)

### check_name
Name of the check that triggered the alarm.

### check_id
ID of the check that triggered the alarm.

### check_error
Error while getting the measurement data, if any.

---

# Alarm
Besides the generic options listed below, alarms have additional options that are specific to check's type.

## Generic options (for all alarm types)
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `"Foobar"` | ❌ | |
| action | `"FooAction"` | ❌ | |
| placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| cycles | `3` | ✔ | `1` |
| repeat_cycles | `100` | ✔ | |
| recover_action | `"FooAction"` | ✔ | |
| recover_placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| recover_cycles | `3` | ✔ | `1` |
| error_action | `"FooAction"` | ✔ | |
| error_placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| error_repeat_cycles | `100` | ✔ | |
| error_recover_action | `"FooAction"` | ✔ | |
| error_recover_placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| invert | `true` | ✔ | `false` |

### disable
If `true`, the alarm is disabled and will not be instantiated.

### name
The name of the alarm. It is used for logging and the `alarm_name` placeholder. Must be unique for the check.

### action
The name of the action to trigger when the state transitions from good to bad.

### placeholders
Custom placeholders that will be merged with the ones of the check and the actions. This one is used for all actions.

### cycles
Number of bad cycles it takes to transition from good to bad state.
Must be at least 1.

### repeat_cycles
If this is non-zero, the action is triggered repeatedly every `repeat_cycles` cycles while in the bad state.
If it is zero, the action is only triggered once when the state transitions from good to bad.

### recover_action
The name of the action to trigger when the state transitions from bad to good.

### recover_placeholders
Custom placeholders that will be merged with the ones of the check and the actions. This one is used only for the `recover_action`.

### recover_cycles
Number of good cycles it takes to transition from bad to good state.
Must be at least 1.

### error_action
The name of the action to trigger when the state transitions from good or bad to error.

### error_placeholders
Custom placeholders that will be merged with the ones of the check and the actions. This one is used only for the `error_action`.

### error_repeat_cycles
If this is non-zero, the action is triggered repeatedly every `error_repeat_cycles` cycles while in the error state.
If it is zero, the action is only triggered once when the state transitions from good or bad to error.

### error_recover_action
The name of the action to trigger when the state transitions from error to good or bad.

### error_recover_placeholders
Custom placeholders that will be merged with the ones of the check and the actions. This one is used only for the `error_recover_action`.

### invert
If `true`, inverts the decision based on the check's measurement data. E.g. the FilesystemUsage check may be used to check if there is **less (or equal)** than 20% of the space used **instead of more** than that.

## Generic placeholders (for all alarm types)

### alarm_name
Name of the alarm that triggered the action.

### alarm_timestamp
ISO8601 timestamp of the alarm's state change event.

### alarm_state
Current state of the alarm.

One of:
- `Good`
- `Bad`
- `Error`
