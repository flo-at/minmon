# FilesystemUsage
Reads the filesystem usage of the given mountpoints.
This check reads the "available blocks" (not "free blocks") i.e. blocks available to unprivileged users.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| mountpoints | `["/srv", "/home"]` | ❌ | | |

### mountpoints
List of mountpoints to check.

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | |

### level
Usage level threshold in percent.
The alarm will be triggered if the measured value exceeds this value.

## IDs
Equivalent to the "mountpoints" config option.

## Placeholders
- `level`: Filesystem space usage (in percent).
