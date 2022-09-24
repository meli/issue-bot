# issue-bot with postfix

Setup your mail server to deliver mail with destination `{local_part}+tags@{domain}` to this binary. Simply call the binary and write the email in UTF-8 in the binary's standard input.

On postfix this can be done by creating a transport map and a pipe. A transport map is a file that tells postfix to send mails send to `{local_part}` to a specific program. The pipe will be this program.

**BEWARE**: If `issue-bot` needs to read its configuration file and database file paths from environment variables, create a wrapper script and call that from postfix instead of going through the complicated trouble of setting up the exported environment (see postfix manual pages `master(t)` and `pipe(8)`)

```shell
/bin/sh

export ISSUE_BOT_CONFIG=_
export ISSUE_BOT_DB=_
/path/to/issue-bot
```

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
