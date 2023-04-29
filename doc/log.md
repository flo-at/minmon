# Log
This configures the logging.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `Warning` | ✔ | `Info` |
| target | `Journal` | ✔ | `Stdout` |

### level
Minimum level for log messages to be sent to the log.

One of:
- `Debug`
- `Info`
- `Warning`
- `Error`

### target
Target the log message is sent to.

One of:
- `Stdout`: Standard output.
- `Stderr`: Standard error.
- `Journal`: Systemd journal (only available if MinMon is built with the `systemd` feature).
