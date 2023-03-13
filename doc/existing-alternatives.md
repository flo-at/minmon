# Existing alternatives
## [Glances](https://nicolargo.github.io/glances/)
Pretty close to what I wanted but:
- Repeated alarms are sent on every check. There is no configuration option to change that.
- There is no action to trigger on recovery.
- [Actions are not triggered in server mode](https://github.com/nicolargo/glances/issues/1879). That's the deal-breaker for my use-case.

## [Netdata](https://www.netdata.cloud/)
Also pretty close to what I wanted. It's "all in one" and easy enough to get it started. Still quite a big tool for such a small task. You explicitly need to disable everything you don't want it to monitor, otherwise it will collect lots of data. They also seem to focus on their cloud service but most things seem to work without it.

## [Monit](https://mmonit.com/monit/)
- Only e-mail alarms.
- Can also run scripts on alarm events but it's not very flexible.
- There's [monit2telegram](https://github.com/matriphe/monit2telegram) to enable basic Telegram notifications.
- Pretty complex config file with it's own scripting language.

## [Uptime Kuma](https://github.com/louislam/uptime-kuma)
This is on the list because I got asked about it a couple of times.
There are some use-cases that overlap between the two but I never really considered Uptime Kuma to be an alternative.
It makes perfect sense to use both at the same time.
MinMon is meant to run on the same machine it's monitoring, so it has direct access to filesystem, memory, and other statistics.
Uptime Kuma on the other hand should ideally run on an external host so you get meaningful downtime statistics which might not be possible if Uptime Kuma itself is down everytime the host it's supposed to monitor is down.

## Others
There are numerous options such as InnoDB/Telegraf, Prometheus, and many more with tons of features and complex configuration that seem overkill for single-instance monitoring/alarming.\
Maybe I've overlooked a tool that meets my requirements. I will stick to (and maintain) MinMon either way because it's very easy to extend with new checks and actions.
