# Report
The report can be triggered on an interval just like the checks. Its main purpose is to let the user know that the service is up and running.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| interval | `60` | ✔ | `604800` |
| cron | `0 * * * * *` | ✔` | |
| placeholders | `{"hostname" = "foobar"}` | ✔ | |
| events | List of [Event](#event) | ✔ | |

### disable
If `true`, the report is disabled and will not be triggered.

### interval
The time between two consecutive reports in seconds.
Set either this or `cron`, but not both.

### cron
Report schedule in [cron-like](https://github.com/zslayton/cron) syntax: `sec  min   hour   day of month   month   day of week   year` where `year` is optional and the local time zone is used.
Set either this or `interval`, but not both.

### placeholders
Custom placeholders that will be merged with ones of the events/actions.

### events
List of [events](#event).

---

# Event
Events configure the relation between the report and the actions.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `FooEvent` | ❌ | |
| action | `FooAction` | ❌ | |
| placeholders | `{"what" = "foobar"}` | ✔ | |

### disable
If `true`, the report is disabled and will not be triggered.

### name
The name of the event. It is used for logging and the `event_name` placeholder.
Must be unique.

### action
The name of the action to trigger when the event is triggered.

### placeholders
Custom placeholders that will be merged with ones of the events/actions.
