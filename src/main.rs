/* This file is part of issue-bot.
 *
 * issue-bot is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * issue-bot is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with issue-bot.  If not, see <https://www.gnu.org/licenses/>.
 */
#![recursion_limit = "1024"]

extern crate log;
extern crate simplelog;
#[macro_use]
extern crate error_chain;

use log::{error, info, trace};
use melib::email::headers::HeaderName;
use melib::{Address, Envelope};
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use simplelog::*;
use std::fs::File;
use std::io::{stdin, Read};
use time::Timespec;
use uuid::Uuid;

mod error;
pub use error::*;
mod api;
mod conf;
use conf::*;
mod cron;
mod templates;

type Password = Uuid;
static PASSWORD_COMMANDS: &'static [&'static str] = &["reply", "unsubscribe", "subscribe", "close"];

#[derive(Debug)]
pub struct Issue {
    id: i64,
    submitter: Address,
    password: Password,
    time_created: Timespec,
    anonymous: bool,
    subscribed: bool,
    title: String,
    last_update: String,
}

pub fn send_mail(d: melib::email::Draft, conf: &Configuration) {
    use std::io::Write;
    use std::process::Stdio;
    let parts = conf.mailer.split_whitespace().collect::<Vec<&str>>();
    let (cmd, args) = (parts[0], &parts[1..]);
    let mut mailer = std::process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to start mailer command");
    {
        let stdin = mailer.stdin.as_mut().expect("failed to open stdin");
        let draft = d.finalise().unwrap();
        stdin
            .write_all(draft.as_bytes())
            .expect("Failed to write to stdin");
    }
    let output = mailer.wait().expect("Failed to wait on mailer");
    if !output.success() {
        // TODO: commit to database queue
        error!("mailer fail");
        eprintln!("mailer fail");
        std::process::exit(1);
    }
}

fn run_app(conn: Connection, conf: Configuration) -> Result<()> {
    let mut new_message_raw = vec![];
    stdin().lock().read_to_end(&mut new_message_raw)?;

    trace!(
        "Received this raw message:\n{}",
        &String::from_utf8_lossy(&new_message_raw)
    );

    let envelope = Envelope::from_bytes(new_message_raw.as_slice(), None)?;
    let mut reply = melib::Draft::new_reply(&envelope, new_message_raw.as_slice(), true);
    reply.headers_mut().insert(
        HeaderName::new_unchecked("From"),
        format!(
            "{local_part}@{domain}",
            local_part = &conf.local_part,
            domain = &conf.domain
        ),
    );

    let tags: Vec<String> = envelope.to()[0].get_tags('+');
    match tags.as_slice() {
        s if s.is_empty() || s == &["anonymous"] => {
            /* Assign new issue */
            let subject = envelope.subject().to_string();
            let body = envelope.body_bytes(new_message_raw.as_slice()).text();
            let from = envelope.from()[0].clone();
            info!("Assign new issue with subject {} from {}", &subject, &from);
            let mut reply = melib::Draft::new_reply(&envelope, new_message_raw.as_slice(), true);
            let anonymous = !tags.is_empty();
            reply.headers_mut().insert(
                HeaderName::new_unchecked("From"),
                format!(
                    "{local_part}@{domain}",
                    local_part = &conf.local_part,
                    domain = &conf.domain
                ),
            );
            match api::new_issue(&conn, subject.clone(), body, anonymous, from, &conf) {
                Ok((password, issue_id)) => {
                    info!("Issue {} successfully created.", &subject);
                    reply.headers_mut().insert(
                        HeaderName::new_unchecked("Subject"),
                        format!(
                            "[{tag}] Issue `{}` successfully created",
                            &subject,
                            tag = &conf.tag
                        ),
                    );
                    reply.set_body(templates::new_issue_success(
                        subject, password, issue_id, &conf,
                    ));
                    send_mail(reply, &conf);
                }
                Err(err) => {
                    error!("Issue {} could not be created {}.", &subject, &err);
                    reply.headers_mut().insert(
                        HeaderName::new_unchecked("Subject"),
                        format!(
                            "[{tag}] Issue `{}` could not be created",
                            &subject,
                            tag = &conf.tag
                        ),
                    );
                    reply.set_body(templates::new_issue_failure(err, &conf));
                    send_mail(reply, &conf);
                }
            }
        }
        &[ref p, ref cmd]
            if Password::parse_str(p).is_ok() && PASSWORD_COMMANDS.contains(&cmd.as_str()) =>
        {
            trace!("Got command {} from {}", cmd.as_str(), &envelope.from()[0]);
            let p = Password::parse_str(p).unwrap();
            match cmd.as_str() {
                "reply" => {
                    info!(
                        "Got reply with subject {} from {}.",
                        &envelope.subject(),
                        &envelope.from()[0]
                    );
                    let body = envelope.body_bytes(new_message_raw.as_slice()).text();
                    let from = envelope.from()[0].clone();
                    match api::new_reply(&conn, body, p, from, &conf) {
                        Ok((title, issue_id, is_subscribed)) => {
                            info!("Reply successfully created.");
                            reply.headers_mut().insert(
                                HeaderName::new_unchecked("Subject"),
                                format!(
                                    "[{tag}] Your reply on issue `{}` has been posted",
                                    &title,
                                    tag = &conf.tag,
                                ),
                            );
                            reply.set_body(templates::new_reply_success(
                                title,
                                p,
                                issue_id,
                                is_subscribed,
                                &conf,
                            ));
                            send_mail(reply, &conf);
                        }
                        Err(err) => {
                            error!(
                                "Reply {} could not be created {}.",
                                &envelope.subject(),
                                &err
                            );
                            reply.headers_mut().insert(HeaderName::new_unchecked("Subject"),
                                    format!(
                                        "[{tag}] Your reply could not be created",
                                        tag = &conf.tag,
                                    ),
                                );
                            reply.set_body(templates::new_reply_failure(err, &conf));
                            send_mail(reply, &conf);
                        }
                    }
                }
                "close" => match api::close(&conn, p, &conf) {
                    Ok((title, issue_id, _)) => {
                        reply.headers_mut().insert(
                            HeaderName::new_unchecked("Subject"),
                            format!(
                                "[{tag}] issue `{}` has been closed",
                                &title,
                                tag = &conf.tag
                            ),
                        );
                        reply.set_body(templates::close_success(title, issue_id, &conf));
                        send_mail(reply, &conf);
                    }
                    Err(e) => {
                        reply.headers_mut().insert(
                            HeaderName::new_unchecked("Subject"),
                            format!("[{tag}] issue could not be closed", tag = &conf.tag,),
                        );
                        reply.set_body(templates::close_failure(e, &conf));
                        send_mail(reply, &conf);
                    }
                },
                "unsubscribe" => match api::change_subscription(&conn, p, false) {
                    Ok((title, issue_id, _)) => {
                        reply.headers_mut().insert(
                            HeaderName::new_unchecked("Subject"),
                            format!(
                                "[{tag}] subscription removal to `{}` successful",
                                &title,
                                tag = &conf.tag
                            ),
                        );
                        reply.set_body(templates::change_subscription_success(
                            title, p, issue_id, false, &conf,
                        ));
                        send_mail(reply, &conf);
                    }
                    Err(e) => {
                        error!("unsubscribe error: {}", e.to_string());
                        reply.headers_mut().insert(
                            HeaderName::new_unchecked("Subject"),
                            format!("[{tag}] could not unsubscribe", tag = &conf.tag,),
                        );
                        reply.set_body(templates::change_subscription_failure(false, &conf));
                        send_mail(reply, &conf);
                    }
                },
                "subscribe" => match api::change_subscription(&conn, p, true) {
                    Ok((title, issue_id, _)) => {
                        reply.headers_mut().insert(
                            HeaderName::new_unchecked("Subject"),
                            format!(
                                "[{tag}] subscription to `{}` successful",
                                &title,
                                tag = &conf.tag
                            ),
                        );
                        reply.set_body(templates::change_subscription_success(
                            title, p, issue_id, true, &conf,
                        ));
                        send_mail(reply, &conf);
                    }
                    Err(e) => {
                        error!("subscribe error: {}", e.to_string());
                        reply.headers_mut().insert(
                            HeaderName::new_unchecked("Subject"),
                            format!("[{tag}] could not subscribe", tag = &conf.tag,),
                        );
                        reply.set_body(templates::change_subscription_failure(true, &conf));
                        send_mail(reply, &conf);
                    }
                },

                other => {
                    reply.headers_mut().insert(
                        HeaderName::new_unchecked("Subject"),
                        format!("[{tag}] invalid action: `{}`", &other, tag = &conf.tag),
                    );
                    reply.set_body(templates::invalid_request(&conf));
                    send_mail(reply, &conf);
                }
            }
        }
        other => {
            reply.headers_mut().insert(
                HeaderName::new_unchecked("Subject"),
                format!("[{tag}] invalid request", tag = &conf.tag),
            );
            reply.set_body(templates::invalid_request(&conf));
            send_mail(reply, &conf);
            error!("invalid request: {:?}", other);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut file = std::fs::File::open("./config.toml")?;
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let perform_cron: bool;
    if args.len() > 1 {
        eprintln!("Too many arguments.");
        std::process::exit(1);
    } else if args == &["cron"] {
        perform_cron = true;
    } else if args.is_empty() {
        perform_cron = false;
    } else {
        eprintln!("Usage: issue_bot [cron]");
        std::process::exit(1);
    }

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let conf: Configuration = toml::from_str(&contents).unwrap();
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Error, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(&conf.log_file).unwrap(),
        ),
    ])
    .unwrap();

    /* - read mail from stdin
     * - decide which case this mail falls to
     *      a) error/junk
     *          reply with error
     *      b) new issue
     *          - assign random id to issue
     *          - reply to sender with id
     *          - save id to sqlite3
     *          - post issue
     *      c) reply
     *      d) close
     *
     *
     */
    let db_path = "./sqlite3.db";
    let conn = Connection::open(db_path).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS issue (
                  id              INTEGER PRIMARY KEY,
                  submitter       TEXT NOT NULL,
                  password        BLOB,
                  time_created    TEXT NOT NULL,
                  anonymous       BOOLEAN,
                  subscribed      BOOLEAN,
                  title           TEXT NOT NULL,
                  last_update     TEXT
                  )",
        NO_PARAMS,
    )
    .unwrap();

    if perform_cron {
        info!("Performing cron duties.");
        if let Err(err) = cron::check(conn, conf) {
            error!("Encountered an error: {}", &err);
            return Err(err);
        }
        return Ok(());
    }

    if let Err(err) = run_app(conn, conf) {
        error!("Encountered an error: {}", &err);
        return Err(err);
    }
    Ok(())
}
