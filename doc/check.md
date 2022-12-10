# Check
Besides the generic options, each check has alarms attached to it.
Some of the type-specific options go into the check's section, some into the alarm's sections - as listed below.

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| interval | `60` | ✔ | `300` |
| name | `Foobar` | ❌ | |
| placeholders | `{"internal_check_id" = "id_foobar"}` | ✔ | |
| type | `FilesystemUsage` | ❌ | |
| alarms | List of [Alarm](#alarm) | ✔ | |

# FilesystemUsage
Reads the filesystem usage of the given mountpoints.
This check reads the "available blocks" (not "free blocks") i.e. blocks available to unprivileged users.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| mountpoints | `["/srv", "/home"]` | ❌ | | |

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | | |

## IDs
Equivalent to the "mountpoints" config option.

## Placeholders
- `level`: Filesystem space usage (in percent).

# MemoryUsage
Reads the system memory (physical RAM) and swap file usage.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| memory | `false` | ✔ | `true` | |
| swap | `true` | ✔ | `false` | |

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| level | `75` | ❌ | | |

## IDs
- `Memory`
- `Swap`

## Placeholders
- `level`: Memory space usage (in percent).

---

# Alarm

## Generic options
| name | example | optional | default |
|:---|:---|:---:|:---|
| disable | `true` | ✔ | `false` |
| name | `Foobar` | ❌ | |
| action | `FooAction` | ❌ | |
| placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| cycles | `3` | ❌ | `1` |
| repeat_cycles | `100` | ✔ | |
| recover_action | `FooAction` | ✔ | |
| recover_placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| recover_cycles | `3` | ✔ | `1` |
| error_action | `FooAction` | ✔ | |
| error_placeholders | `{"internal_alarm_id" = "id_foobar"}` | ✔ | |
| error_repeat_cycles | `100` | ✔ | |
| invert | `true` | ✔ | `false` |
