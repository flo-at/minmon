# Webhook
Trigger a Webhook.

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| url | `"http://example.com/webhook?alarm={{alarm_name}}"` | ❌ | | ✔ |
| method | `"GET"`, `"POST"`, `"PUT"`, `"DELETE"`, `"PATCH"` | ✔ | `"POST"` | ❌ |
| headers | `{"Content-Type" = "application/json"}` | ✔ | | ❌ |
| body | `{"text": "Triggered from check '{{check_name}}'."}`  | ✔ | | ✔ |

### url
URL the HTTP request will be sent to.

### method
HTTP method used for the request.

### headers
HTTP headers used for the request

### body
HTTP request body.
