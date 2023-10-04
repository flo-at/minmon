# ProcessExitStatus
Runs a process and checks its exit status code.

## Check options
| name | example | optional | default |
|:---|:---|:---:|:---|
| path | `"/usr/bin/echo"` | ❌ | |
| arguments | `["-e", "Checking things.."]` | ✔ | |
| environment_variables | `{"FOO": "BAR"}` | ✔ | |
| working_directory | `"/home/user/"` | ✔ | inherited (\*) |
| uid | `1000` | ✔ | inherited (*) |
| gid | `1000` | ✔ | inherited (*) |
| stdout_max | `256` | ✔ | 512 |
| stderr_max | `256` | ✔ | 512 |

(\*) Inherited from MinMon's process.

### path
Absolute path to the executable to be called.

### arguments
List of arguments to be passed to the process.

### environment_variables
Environment variables to be set in the process environment.

### working_directory
Working directory for the spawned process.
The new process will inherit MinMon's working directory if this is not set.

### uid
User ID the process will be run with.
The new process will inherit MinMon's user ID if this is not set.

### gid
Group ID the process will be run with.
The new process will inherit MinMon's group ID if this is not set.

### stdout_max
Maximum number of bytes read from standard output.

### stderr_max
Maximum number of bytes read from standard error.

## Alarm options
| name | example | optional | default |
|:---|:---|:---:|:---|
| status_codes | `[1, 255]` | ✔ | `[0]` |

### status_codes
List of all possible "good" process exit status codes.
All other exist status codes will be considered "bad".

## IDs
Name of the file given by the path.

## Placeholders
- `status_code`: Process exit status code.
