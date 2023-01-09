# MemoryUsage
Reads the system memory (physical RAM) and swap file usage.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| memory | `true` | ✔ | `false` | |
| swap | `true` | ✔ | `false` | |

### memory
If `true`, physical RAM usage will be checked.
The measured value will have the ID `Memory`.

### swap
If `true`, swap space usage will be checked.
The measured value will have the ID `Swap`.

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | |

### level
Usage level threshold in percent.
The alarm will be triggered if the measured value exceeds this value.

## IDs
- `Memory`
- `Swap`

## Placeholders
- `level`: Memory space usage (in percent).
