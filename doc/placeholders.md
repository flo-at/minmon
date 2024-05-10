# Generic placeholders (always available)

## system_uptime
System uptime in seconds.

## system_uptime_iso
System uptime as ISO8601 duration.
This does not use the month and year fields because they are ambiguous.

## minmon_uptime
MinMon's uptime in seconds.

## minmon_uptime_iso
MinMon's uptime as ISO8601 duration.
This does not use the month and year fields because they are ambiguous.

## env:MINMON_HELLO
Value of the environment variable `MINMON_HELLO`.
Only variables that match the prefix configured in the general config section are evaluated.
