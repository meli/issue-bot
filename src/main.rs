use melib::Envelope;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use std::io::{stdin, Read};
use time::Timespec;
use uuid::Uuid;

mod error;
use error::*;
mod api;
mod conf;
use conf::*;
mod cron;
mod templates;

type Password = Uuid;
static PASSWORD_COMMANDS: &'static [&'static str] = &["reply", "unsubscribe", "subscribe", "close"];

use melib::{email::parser, Address};

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

fn new_address(s: &str) -> Address {
    parser::address(s.as_bytes()).to_full_result().unwrap()
}

pub fn send_mail(d: melib::email::Draft, conf: &Config) {
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
        eprintln!("mailer fail");
        std::process::exit(1);
    }
}

fn main() -> std::result::Result<(), std::io::Error> {
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
    let conf: Config = toml::from_str(&contents).unwrap();
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
        cron::check(conn, conf);
        return Ok(());
    }

    let mut new_message_raw = String::new();
    stdin().lock().read_to_string(&mut new_message_raw).unwrap();

    let envelope = Envelope::from_bytes(new_message_raw.as_bytes(), None);
    if let Ok(envelope) = envelope {
        let mut reply = melib::Draft::new_reply(&envelope, &[]);
        reply.headers_mut().insert(
            "From".to_string(),
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
                let body = envelope.body_bytes(new_message_raw.as_bytes()).text();
                let from = envelope.from()[0].clone();
                let mut reply = melib::Draft::new_reply(&envelope, &[]);
                let anonymous = !tags.is_empty();
                reply.headers_mut().insert(
                    "From".to_string(),
                    format!(
                        "{local_part}@{domain}",
                        local_part = &conf.local_part,
                        domain = &conf.domain
                    ),
                );
                match api::new_issue(&conn, subject.clone(), body, anonymous, from, &conf) {
                    Ok((password, issue_id)) => {
                        reply.headers_mut().insert(
                            "Subject".to_string(),
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
                    Err(e) => {
                        reply.headers_mut().insert(
                            "Subject".to_string(),
                            format!(
                                "[{tag}] Issue `{}` could not be created",
                                &subject,
                                tag = &conf.tag
                            ),
                        );
                        reply.set_body(templates::new_issue_failure(e, &conf));
                        send_mail(reply, &conf);
                    }
                }
            }
            &[ref p, ref cmd]
                if Password::parse_str(p).is_ok() && PASSWORD_COMMANDS.contains(&cmd.as_str()) =>
            {
                let p = Password::parse_str(p).unwrap();
                match cmd.as_str() {
                    "reply" => {
                        let body = envelope.body_bytes(new_message_raw.as_bytes()).text();
                        let from = envelope.from()[0].clone();
                        match api::new_reply(&conn, body, p, from, &conf) {
                            Ok((title, issue_id, is_subscribed)) => {
                                reply.headers_mut().insert(
                                    "Subject".to_string(),
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
                            Err(e) => {
                                reply.headers_mut().insert(
                                    "Subject".to_string(),
                                    format!(
                                        "[{tag}] Your reply could not be created",
                                        tag = &conf.tag,
                                    ),
                                );
                                reply.set_body(templates::new_reply_failure(e, &conf));
                                send_mail(reply, &conf);
                            }
                        }
                    }
                    "close" => match api::close(&conn, p, &conf) {
                        Ok((title, issue_id, _)) => {
                            reply.headers_mut().insert(
                                "Subject".to_string(),
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
                                "Subject".to_string(),
                                format!("[{tag}] issue could not be closed", tag = &conf.tag,),
                            );
                            reply.set_body(templates::close_failure(e, &conf));
                            send_mail(reply, &conf);
                        }
                    },
                    "unsubscribe" => match api::change_subscription(&conn, p, false) {
                        Ok((title, issue_id, _)) => {
                            reply.headers_mut().insert(
                                "Subject".to_string(),
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
                            eprintln!("error: {}", e.to_string());
                            reply.headers_mut().insert(
                                "Subject".to_string(),
                                format!("[{tag}] could not unsubscribe", tag = &conf.tag,),
                            );
                            reply.set_body(templates::change_subscription_failure(false, &conf));
                            send_mail(reply, &conf);
                        }
                    },
                    "subscribe" => match api::change_subscription(&conn, p, true) {
                        Ok((title, issue_id, _)) => {
                            reply.headers_mut().insert(
                                "Subject".to_string(),
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
                            eprintln!("error: {}", e.to_string());
                            reply.headers_mut().insert(
                                "Subject".to_string(),
                                format!("[{tag}] could not subscribe", tag = &conf.tag,),
                            );
                            reply.set_body(templates::change_subscription_failure(true, &conf));
                            send_mail(reply, &conf);
                        }
                    },

                    other => {
                        reply.headers_mut().insert(
                            "Subject".to_string(),
                            format!("[{tag}] invalid action: `{}`", &other, tag = &conf.tag),
                        );
                        reply.set_body(templates::invalid_request(&conf));
                        send_mail(reply, &conf);
                    }
                }
            }
            other => {
                reply.headers_mut().insert(
                    "Subject".to_string(),
                    format!("[{tag}] invalid request", tag = &conf.tag),
                );
                reply.set_body(templates::invalid_request(&conf));
                send_mail(reply, &conf);
                println!("error: {:?}", other);
            }
        }
    }

    Ok(())
}
