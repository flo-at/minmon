# General
This configures the general behaviour of MinMon.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| boot_delay | `60` | ✔ | |
| start_delay | `10` | ✔ | |
| env_var_prefix | `FOO_` | ✔ | `MINMON_` |

### boot_delay
The minimum system uptime (in seconds) MinMon awaits when it starts before the checks begin.
Use this to give the monitored services some time to start after the system booted.

### start_delay
MinMon waits this many seconds when it starts before the checks begin.

### env_var_prefix
Prefix of environment variables that should be available as placeholders in the form of
`{{env:MINMON_HELLO}}`.
