# Process
Call a process.

## Options
| name | example | optional | default | placeholders |
|:---|:---|:---:|:---|:---:|
| path | `"/usr/bin/echo"` | ❌ | | ❌ |
| arguments | `["-e", "Alarm '{{alarm_name}}' was triggered."]` | ✔ | | ✔ |
| environment_variables | `{"ALARM_NAME": "{{alarm_name}}"}` | ✔ | | ✔ |
| working_directory | `"/home/user/"` | ✔ | inherited (\*) | ❌ |
| uid | `1000` | ✔ | inherited (*) | ❌ |
| gid | `1000` | ✔ | inherited (*) | ❌ |

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
