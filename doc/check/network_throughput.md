# NetworkThroughput
Reads the number of bytes sent and/or received on a network interface.\
Will always report `0` units when triggered for the first time.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| interfaces | `["enp0s1", "wg0"]` | ❌ | |
| received | `true` | ✔ | `false` |
| sent | `true` | ✔ | `false` |
| log_format | `"Decimal"` | ✔ | `"Binary"` |

### interfaces
List of network interfaces to check.

### received
Check received/incoming data.

### sent
Check sent/outgoing data.

### log_format
Formatting of the throughput data size for the log output.

One of:
- `"Binary"`: Powers of 1024 (KiB, MiB, GiB), max. precision 3
- `"Decimal"`: Powers of 1000 (kB, MB, GB), max. precision 3
- `"Bytes"`: Number of bytes

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| data_size | `100` | ❌ | |
| unit | `"Kilobyte"` | ✔ | `"Byte"` |

### data_size
Amount of units sent/received since the last check interval.
The alarm will be triggered if the measured value exceeds this value.

### unit
Unit of the value in "data_size".

One of:
- `"Byte"`
- `"Kilobyte"`
- `"Megabyte"`
- `"Gigabyte"`
- `"Kibibyte"`
- `"Mebibyte"`
- `"Gibibyte"`

See [Wikipedia](https://en.wikipedia.org/wiki/Byte#Multiple-byte_units) for more information.

## IDs
Equivalent to the "interfaces" config option with "[rx]" or "[tx]" suffix (e.g. `enp0s1[rx]`).

## Placeholders
- `data_size`: Measured data throughput since last check interval (in bytes).
- `data_size_bin`: Measured data throughput since last check interval (bytes in powers of 1024).
- `data_size_dec`: Measured data throughput since last check interval (bytes in powers of 1000).
