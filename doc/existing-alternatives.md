# Existing alternatives
## [Glances](https://nicolargo.github.io/glances/)
Pretty close to what I wanted but:
- Repeated alarms are sent on every check. There is no configuration option to change that.
- There is no action to trigger on recovery.
- [Actions are not triggered in server mode](https://github.com/nicolargo/glances/issues/1879). That's the deal-breaker for my use-case.

## [Netdata](https://www.netdata.cloud/)
Also pretty close to what I wanted. It's "all in one" and easy enough to get it started. Still quite a big tool for such a small task.

## [Monit](https://mmonit.com/monit/)
- Only e-mail alarms.
- Can also run scripts on alarm events but it's not very flexible.
- There's [monit2telegram](https://github.com/matriphe/monit2telegram) to enable basic Telegram notifications.
- Pretty complex config file with it's own scripting language.

## Others
There are numerous options such as InnoDB/Telegraf, Prometheus, and many more with tons of features and complex configuration that seem overkill for single-instance monitoring/alarming.
