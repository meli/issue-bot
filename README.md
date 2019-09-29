# `issue_bot`

A bot to handle bug reports through mail, for gitea's issue tracker.

## Problem

Users have to register to your gitea instance to file bugs. This is a deterrent. A mailing list requires less effort, but lacks bridging with an issue tracker.

## Solution

Users send new issues with an e-mail to the address of your bot. Your bot replies with a password that allows the author to reply with the same identity and to close the issue.

The bot binary can also be run periodically to check for new replies in issues and send the updates to the issue authors, if they are subscribed to the issue. Subscription is true by default, and the subscription status can be changed with the password.

## Problems this solution brings

Spam?

## Configuration

The bot looks for a `config.toml` file in the same directory as the binary. The config needs the following values:

```text
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

Setup your mail server to deliver mail with destination `{local_part}+tags@{domain}` to this binary. Simply call the binary and write the email in UTF-8 in the binary's standard input.

On postfix this can be done by creating a transport map and a pipe. A transport map is a file that tells postfix to send mails send to `{local_part}` to a specific program. The pipe will be this program.

Open `master.cf` and paste this line at the bottom: 

```text
issue_bot unix - n n - - pipe
  user=issuebot directory=/path/to/binarydir/ argv=/path/to/binary
```

an example:


```text
issue_bot unix - n n - - pipe
  user=issuebot directory=/home/issuebot/ argv=/home/issuebot/issue-bot
```

Then create your transport map:

```text
{local_part}@{domain} issue_bot:
```

Notice the colon at the end. This means that it refers to a transfer, not an address. Save the file somewhere (eg `/etc/postfix/issue_transport`) and make it readable by postfix. Issue `postmap /etcpostfix/issue_transport`. Finally add the entry `hash:/etc/postfix/issue_transport` in your `transport_maps` and `local_recipient_maps` keys in `main.cf`. `postfix reload` to load the configuration changes.

You will also need the following setting to allow tags in your recipient addresses:

```text
recipient_delimiter = +
```

Setup a periodic check in your preferred task scheduler to run `issue_bot_bin cron` in order to fetch replies to issues. On systemd this can be done with timers.


### Troubleshooting
If you your email stops working or postfix doesn't pass mail to the bot, make sure you're not using a non-default setup like virtual mailboxes. In that case you have to add the transport along with the transports of your setup, whatever that be.

If the e-mail gets to the binary and nothing happens, make sure:

- the binary is executable and readable by the pipe's user
- the configuration file is in the same directory as the binary
- that in `master.cf` there are no `flags=` in the transport entry. The mail must be piped unaltered.
- your auth token works. You can check yourself by issuing requests to your API via cURL. There are examples here: https://docs.gitea.io/en-us/api-usage/

If commands (using +reply, +close etc) don't work, make sure you have added `recipient_delimiter = +` in your `main.cf` file.

The bot's state is saved in a sqlite3 database in the same directory as the binary. You can view its data by using the `sqlite3` cli tool:

```shell
root# sqlite3 /home/issuebot/sqlite3.db
SQLite version ****** ********** ********
Enter ".help" for usage hints.
sqlite> .tables
issue
sqlite> select * from issue;
1|Name <add@res.tld>|1F:|2019-09-29T12:20:21.658495173Z|0|1|issue title|"2019-09-29T15:20:21+03:00"
2|Name <add@res.tld>|{^D0u|2019-09-29T12:23:48.291970808Z|0|1|issue title#2|"2019-09-29T15:23:48+03:00"
3|Name <add@res.tld>|Gd)i]|2019-09-29T12:24:31.414792595Z|0|1|issue title again|"2019-09-29T15:26:53+03:00"
4|Name <add@res.tld>|$3fBוv|2019-09-29T12:28:21.187425505Z|1|1|many issues|"2019-09-29T15:28:21+03:00"
```
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
