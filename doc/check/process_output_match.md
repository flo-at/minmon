# ProcessrOutputMatch
Runs a process and matches its output agains a regular expression.
The output is read from either stdout or stderr.

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
| output_source | `"Stderr"` | ✔ | `"Stdout"` |
| output_regex | `'^Value: (\d+)$'` | ❌ | |
| invert_match | `true` | ✔ | `false` |

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

### output_source
Process output source.

One of:       
- `"Stdout"`    
- `"Stderr"`

### output_regex
Regular expression to match the output.
Use TOML's literal strings (single-quoted) for regular expressions so you don't have to escape backslashes all the time.

### invert_match
If this is `false`, a regex match is considered "good", else "bad" and the other way around.

## Alarm options
None.

## IDs
Name of the file given by the path.

## Placeholders
- `capture[n]`: Process output value of the n-th capture group of `output_regex`, starting at 0 and including the implicit unnamed group that corresponds to the entire match.
- `stdout`: Text read from process standard output without leading and trailing whitespace.
- `stderr`: Text read from process standard error without leading and trailing whitespace.
