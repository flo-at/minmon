# DockerContainerStatus
Checks whether a docker container is running (and healthy).\
This check is only available if MinMon is built with the `docker` feature.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| socket_path | `/srv/docker.sock` | ✔ | `/var/run/docker.sock` |
| containers | `["foo", "bar"]` | ❌ | |

### socket_path
Path to the docker daemon's UNIX socket.

### containers
List of container names to be checked.

## Alarm options
None.

## IDs
Container names.

## Placeholders
- `state`: `true` if container is running (and healthy) else `false`.
