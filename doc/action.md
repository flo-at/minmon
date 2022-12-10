# Action
Actions control what happens when an alarm or report event is triggered.

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `Foobar` | ❌ | |
| placeholders | `{"internal_action_id" = "id_foobar"}` | ✔ | |
| type | `Log` | ❌ | |

# Log
Write a line to the log (as configured in the `[log]` section of the config file).

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| level | `Debug`, `Info`, `Warning`, `Error` | ✔ | `Info` | ❌ |
| template | `Alarm '{{alarm_name}}' was triggered.` | ❌ | | ✔ |

# Process
Call a process.

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| path | `/usr/bin/echo` | ❌ | | ❌ |
| arguments | `["-e", "Alarm '{{alarm_name}}' was triggered."]` | ✔ | | ✔ |
| environment_variables | `{"ALARM_NAME": "{{alarm_name}}"}` | ✔ | | ✔ |
| working_directory | `/home/user/` | ✔ | Inherited (\*) | ❌ |
| uid | `1000` | ✔ | Inherited (*) | ❌ |
| gid | `1000` | ✔ | Inherited (*) | ❌ |

(\*) Inherited from MinMon's process.

# Webhook
Trigger a Webhook.

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| url | `http://example.com/webhook?alarm={{alarm_name}}` | ❌ | | ✔ |
| method | `GET`, `POST`, `PUT`, `DELETE`, `PATCH` | ✔ | `POST` | ❌ |
| headers | `{"Content-Type" = "application/json"}` | ✔ | | ❌ |
| timeout | `3` | ✔ | `10` | ❌ |
| body | `{"text": "Triggered from check '{{check_name}}'."}`  | ✔ | | ✔ |
