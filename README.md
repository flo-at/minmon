# MinMon - opinionated minimal monitoring and alarming tool
This tool is just a single binary and a config file. No database, no GUI, no graphs, no runtime dependencies. Just monitoring and alarms.
I wrote this because the [exsiting alternatives](#existing-alternatives) I could find were too heavy, mainly focused on nice GUIs with graphs (not on alarming), too complex to setup or targeted at huge multi-instance setups.

# Design decisions
- No complex scripting language.
- No fancy config directory structure - just a single TOML file.
- No cryptic abbreviations. The few extra letters in the config file won't hurt anyone.
- There are no predefined threshold names like "Warning" or "Critical". You might might want more than just two, or only one. So that's up to you to define in the config.
- The same check plugin can be used multiple times. You might want different levels to trigger different actions for different filesystems/storages/.. (TODO try to batch queries to reduce monitoring load)
- Alarms are timed in "cycles" (i.e. multiples of the `interval` of the check) instead of seconds. It's not very user-friendly but helps keep the internal processing simple and efficient.
- Alarms stand for themselves - they are not related. This means that depending on your configuration, two (or more) events may be triggered at the same time for the same check. There are cases where this is not desired.
- Simple, clean, bloat-free code.
- Depending on your configuration, there might be similar or identical blocks in the config file. This is a consequence of the flexibility and simpleness of the config file format (and thus the code).
- All times and dates are UTC. No fiddling with local times and time zones.

# Checks
- Filesystem usage
- Memory

## Ideas
- Filesystem inode usage
- Folder size
- S.M.A.R.T.
- Load
- Temperature
- Ping
- HTTP

# Actions
- WebHook

## Ideas
- E-mail

# Report
The absence of alarms can mean two things: everything is okay or the monitoring/alarming failed.
That's why MinMon can trigger regular report actions to let you know that it's up and running.

# Existing alternatives

## [Glances](https://nicolargo.github.io/glances/)
Closest to what I wanted but:
- Repeated alarms are sent on every check. There is no configuration option to change that.
- There is no action to trigger on recovery.
- [Actions are not triggered in server mode](https://github.com/nicolargo/glances/issues/1879). That's a deal-breaker.

## [Netdata](https://www.netdata.cloud/)
- It's "all in one" and easy enough to get it started. Still quite a big tool for such a small task.

## [Monit](https://mmonit.com/monit/)
- Only e-mail alarms.
- Can also run scripts on alarm events but it's not very flexible.
- There's [monit2telegram](https://github.com/matriphe/monit2telegram) to enable basic Telegram notifications.
- Pretty complex config file with it's own scripting language.

## Others
There are numerous options such as InnoDB/Telegraf, Prometheus, and many more with tons of features and complex configuration that seem overkill for single-instance monitoring/alarming.

# Future ideas
- Write to RRD-file + "Uptime Kuma"-like history (in a separate project/application): ncurses or web
