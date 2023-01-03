# Peak
Calculates the peak value of a moving window buffer of measurement values.
The window buffer grows until it reaches its final size.
Measurement values that are missing due to an error in the check reduce the effective window size until they are moved out again.

## Options
| name | example | optional | default |
|:---|:---|:---:|:---|
| window_size | `16` | ❌ | |

### window_size
Size of the moving window.
