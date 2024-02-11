# DockerContainerStatus
Checks whether a docker container is running (and healthy).\
This check is only available if MinMon is built with the `docker` feature.

If MinMon is running inside a docker container, the docker socket has to be mounted for this check to work.\
This can be done by adding the argument `-v /var/run/docker.sock:/var/run/docker.sock` to the docker run command.

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
