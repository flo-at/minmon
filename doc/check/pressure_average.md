# PressureAverage
Reads the average values from the Linux kernel's Pressure Stall Information (PSI).
See the [kernel documentation](https://www.kernel.org/doc/html/latest/accounting/psi.html) for more information.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| cpu | `true` | ✔ | `false` | |
| io | `"None"`, `"Some"`, `"Full"`, `"Both"` | ✔ | `"None"` | |
| memory | `"None"`, `"Some"`, `"Full"`, `"Both"` | ✔ | `"None"` | |
| avg10 | `true` | ✔ | `false` | |
| avg60 | `true` | ✔ | `false` | |
| avg300 | `true` | ✔ | `false` | |

### cpu
If `true`, the CPU pressure level will be checked.

### io
If `true`, the I/O pressure level will be checked.

### memory
If `true`, the memory pressure level will be checked.

### avg10
If `true`, the 10 second average pressure level will be checked.

### avg60
If `true`, the 1 minute average pressure level will be checked.

### avg300
If `true`, the 5 minute average pressure level will be checked.

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | | |

### level
Pressure average level threshold in percent.
The alarm will be triggered if the measured value exceeds this value.

## IDs
- All combinations of `cpu/{avg10,avg60,avg300}`
- All combinations of `{io,memory}/{some,full}/{avg10,avg60,avg300}`

## Placeholders
- `level`: Pressure average (in percent).
