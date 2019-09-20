# issue_bot

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
auth_token= "Basic ________________________________________________________________________________________________________________________________________________________________"
# the local part of your bot's receiving address.
local_part= "issues"
domain= "meli.delivery"
base_url = "https://git.meli.delivery"
repo = "meli/issue-bot"
bot_name = "IssueBot"
bot_username = "issue_bot"
# the shell command that the bot pipes mail to
mailer = "cat"
```

Setup your mail server to deliver mail with destination `{local_part}+tags@{domain}` to this binary. Simply call the binary and write the email in UTF-8 in the binary's standard input.

Setup a periodic check in your preferred task scheduler to run `issue_bot_bin cron` in order to fetch replies to issues.

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
