# SystemdUnitStatus
Checks whether a systemd unit is active or not.\
This check is available even if MinMon is built without the `systemd` feature.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| units | `["dbus.service", {unit = "foo.service", uid = 1000}]` | ❌ | |

### units
List of systemd units to check.
Each entry can be either a string that is the name of a unit or an object with the unit's name and the user's UID.
If the UID is non-zero, `systemctl --user` will be run with the given UID.

## Alarm options
None.

## IDs
Unit names with UIDs (if non-zero) (e.g. `foo.service[1000]`).

## Placeholders
- `state`: `true` if service is active else `false`.
