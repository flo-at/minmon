# Report
The report can be triggered on an interval just like the checks. It's main purpose is to let the user know that the service is up and running.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| interval | `60` | ✔ | `604800` |
| placeholders | `{"hostname" = "foobar"}` | ✔ | |
| events | List of [Event](#event) | ✔ | |

---

# Event
Events configure the connection between the report and the actions.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `FooEvent` | ❌ | |
| action | `FooAction` | ❌ | |
| placeholders | `{"what" = "foobar"}` | ✔ | |
