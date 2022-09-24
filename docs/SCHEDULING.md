# issue-bot scheduled jobs

You can set up scheduled jobs by configuring `crontab` to run `issue-bot cron` whenever you want. For the more complicated but more reliable systemd setup, two example files are included in this directory: a service unit file and a timer unit file. The service unit executes once, and the timer unit is responsible for calling the service at the intervals you set.

Copy the example files somewhere else and edit them with your own values.

You can put `dry_run = true` in the config file to check it works without making changes or sending any mail. Also, backup your database if needed.

```shell
systemctl --user enable issue-bot.service
```

You can do a test run with

```shell
systemctl --user start issue-bot.service
```

Now you enable/activate the timer.

```shell
systemctl --user enable issue-bot.timer
```

```shell
systemctl --user start issue-bot.timer
```


Monitor the service status:

```shell
systemctl --user status issue-bot
```
