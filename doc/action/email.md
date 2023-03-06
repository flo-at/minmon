# Email
Send an email.
This action is only available if MinMon is built with the `smtp` feature.

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| from | `"foo@example.com"` | ❌ | | ❌ |
| to | `"bar@example.com"` | ❌ | | ❌ |
| reply_to | `"noreply@example.com"` | ✔ | | ❌ |
| subject | `"Alarm from check '{{check_name}}'!"` | ❌ | | ✔ |
| body | `"Check '{{check_name}}' is not happy!"` | ❌ | | ✔ |
| smtp_server | `"smtp.example.com"` | ❌ | | ❌ |
| smtp_port | `587` | ✔ | auto | ❌ |
| smtp_security | `"TLS"`, `"STARTTLS"`, `"Plain"` | ✔ | `"TLS"` | ❌ |
| username | `"johndoe"` | ❌ | | ❌ |
| password | `"topsecret"` | ❌ | | ❌ |

### from
Email address of the sender.

### to
Email adddress of the recipient.

### reply_to
Email address the recipient should reply to.

### subject
Subject of the email.

### body
Body of the email.

### smtp_server
Hostname of the SMTP server.

### smtp_port
Port of the SMTP server.

### smtp_security
SMTP security mode to use for the connection.

### username
Username of the sender's account on the SMTP server.

### password
Password of the sender's account on the SMTP server.
