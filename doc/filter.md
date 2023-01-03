# Filter
Filters define transformations that can be applied to measurement data.
Besides the generic options listed below, filters have additional options that are specific to their type.
They can be applied to checks and actions.

There are some combinations that won't work because they don't make any sense.
For example, you cannot sum up temperatures and there's no average process exit status code.
MinMon will verify this when it starts an lets you know if something is wrong here.

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| type | `"Average"` | ❌ | |

### type
Type of the filter as listed below.

One of:
- [Average](./filter/average.md)
- [Peak](./filter/peak.md)
- [Sum](./filter/sum.md)
