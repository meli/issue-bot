# `issue_bot`

A bot to handle bug reports through mail, for gitea's issue tracker.

Expects configuration file path in environment variable `ISSUE_BOT_CONFIG`. If
it's not defined, default is `./config.toml` (current working directory).

Expects database file path in environment variable `ISSUE_BOT_DB`. If it's not
defined, default is `./sqlite3.db` (current working directory).

```
issue-bot
```

by default expects to read a valid [RFC5322](https://www.rfc-editor.org/rfc/rfc5322) e-mail in valid utf-8 or valid ascii.


```
issue-bot cron
```

Checks if there are new comments or other updates in the issues, and sends
emails to anyone subscribed. An example systemd service and timer file is provided in `docs/`.

## Problem

Users have to register to your gitea instance to file bugs. This is a deterrent. A mailing list requires less effort, but lacks bridging with an issue tracker.

## Solution

Users send new issues with an e-mail to the address of your bot. Your bot replies with a password that allows the author to reply with the same identity and to close the issue.

The bot binary can also be run periodically to check for new replies in issues and send the updates to the issue authors, if they are subscribed to the issue. Subscription is true by default, and the subscription status can be changed with the password.

## Problems this solution brings

Spam?

## Configuration

The config file must be valid TOML and needs the following values:

```toml
# the tag that prefixes email subjects eg "[issue-bot-issues] blah blah"
tag = "issue-bot-issues"
# the auth_token for the gitea's instance API
auth_token= "7a36300555aaf84a0af9847e9747a0df72a82056"
# the local part of your bot's receiving address.
local_part= "issues"
domain= "meli.delivery"
base_url = "https://git.meli.delivery"
repo = "meli/issue-bot"
bot_name = "IssueBot"
bot_username = "issue_bot"
# the shell command that the bot pipes mail to
# mailer = "cat" # just print the e-mail in stdout 
# mailer = "/usr/sbin/sendmail -t webmaster@meli.delivery" # send copies to an address
mailer = "/usr/sbin/sendmail -t"
```

Optionally, you can set `dry_run = true` to avoid any email/db update being performed in order to debug what would happen if you ran the `cron` command.

Setup your mail server to deliver mail with destination `{local_part}+tags@{domain}` to this binary. Simply call the binary and write the email in UTF-8 in the binary's standard input.

For postfix setup see `docs/POSTFIX.md`.

## Demo

My email:

```e-mail
Date: Fri, 20 Sep 2019 20:12:21 +0300
From: me@domain.tld
To: issues@git.tld
Subject: Issue title

Issue body text, with formatting.
```

The reply I get:

```e-mail
Date: Fri, 20 Sep 2019 20:12:21 +0300
From: issues@git.tld
To: me@domain.tld
<--8<--->
Subject: [issue-bot-issues] Issue `Issue title` successfully created

Hello,

You have successfully submitted an issue titled "Issue title". Your issue can be found at

https://git.tld/epilys/test/issues/24

You will receive replies from other users. To unsubscribe from the conversation, send an email to issues+5590e09e-b6da-419b-b8d9-e86852d8d6b1+unsubscribe@git.tld.

To reply to other users or post new comments, send your text to issues+5590e09e-b6da-419b-b8d9-e86852d8d6b1+reply@git.tld.

To close the issue, send an email to issues+5590e09e-b6da-419b-b8d9-e86852d8d6b1+close@git.tld.

Please keep this email in order to be able to keep in touch with your issue.

This is an automated email from IssueBot <issues+help@git.tld>
```

A reply notice sent for subscribed issues:

```e-mail
Date: Fri, 20 Sep 2019 21:06:56 +0300
From: issues@git.tld
To: me@domain.tld
Subject: [issue-bot-issues] new replies in issue `Issue title`

Hello,

There have been new replies in issue `Issue title`. You are receiving this notice because you are subscribed to the discussion. To unsubscribe, send an email to issues+5590e09e-b6da-419b-b8d9-e86852d8d6b1+unsubscribe@git.tld

me@domain.tld replies:

This is my reply

This is an automated email from IssueBot <issues+help@git.tld>
```
