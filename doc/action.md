# Action
Actions control what happens when an alarm or report event is triggered.

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `Foobar` | ❌ | |
| timeout | `3` | ✔ | `10` | ❌ |
| placeholders | `{"internal_action_id" = "id_foobar"}` | ✔ | |
| type | `Email` | ❌ | |

# Email
Send an email.

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| from | `foo@example.com` | ❌ | | ❌ |
| to | `bar@example.com` | ❌ | | ❌ |
| reply_to | `noreply@example.com` | ✔ | | ❌ |
| subject | `Alarm from check '{{check_name}}'!` | ❌ | | ✔ |
| body | `Check '{{check_name}}' is not happy!` | ❌ | | ✔ |
| smtp_server | `smtp.example.com` | ❌ | | ❌ |
| smtp_port | `587` | ✔ | auto | ❌ |
| smtp_security | `TLS`, `STARTTLS`, `Plain` | ✔ | `TLS` | ❌ |
| username | `johndoe` | ❌ | | ❌ |
| password | `topsecret` | ❌ | | ❌ |

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
| working_directory | `/home/user/` | ✔ | inherited (\*) | ❌ |
| uid | `1000` | ✔ | inherited (*) | ❌ |
| gid | `1000` | ✔ | inherited (*) | ❌ |

(\*) Inherited from MinMon's process.

# Webhook
Trigger a Webhook.

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| url | `http://example.com/webhook?alarm={{alarm_name}}` | ❌ | | ✔ |
| method | `GET`, `POST`, `PUT`, `DELETE`, `PATCH` | ✔ | `POST` | ❌ |
| headers | `{"Content-Type" = "application/json"}` | ✔ | | ❌ |
| body | `{"text": "Triggered from check '{{check_name}}'."}`  | ✔ | | ✔ |
