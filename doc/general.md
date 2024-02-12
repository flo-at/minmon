# General
This configures the general behaviour of MinMon.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| boot_delay | `60` | ✔ | |
| start_delay | `10` | ✔ | |

### boot_delay
The minimum system uptime (in seconds) MinMon awaits when it starts before the checks begin.
Use this to give the monitored services some time to start after the system booted.

### start_delay
MinMon waits this many seconds when it starts before the checks begin.
