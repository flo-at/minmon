# Action
Besides the generic options listed below, actions have additional options that are specific to their type.
Actions control what happens when an alarm (check) or event (report) is triggered.

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `"Foobar"` | ❌ | |
| timeout | `3` | ✔ | `10` | ❌ |
| placeholders | `{"internal_action_id" = "id_foobar"}` | ✔ | |
| type | `"Email"` | ❌ | |

### disable
If `true`, the action is disabled and will not be triggered.

### name
The name of the action. It is used for logging and the `action_name` placeholder.
Must be unique.

### timeout
The maximum time in seconds an action may take to finish its execution before being interrupted.

### placeholders
Custom placeholders that will be merged with ones of the check/alarm.

### type
Type of the check as listed below.
This determines which specific check and alarm options are available.

One of:
- [Email](./action/email.md)
- [Log](./action/log.md)
- [Process](./action/process.md)
- [Webhook](./action/webhook.md)

## Generic placeholders (for all action types)

### action_name
Name of the action that was triggered.
