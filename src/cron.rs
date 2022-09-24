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

use super::*;
use melib::email::address::Address;

pub fn check_issue(conn: &Connection, conf: &Configuration, issue: Issue) -> Result<bool> {
    let mut update = false;
    let mut comments = api::comments(issue.id, &issue.last_update, conf)?;
    let mut new_value = issue.last_update.clone();
    for c in &comments {
        _ = gitea_api_mismatch!(c["created_at"].as_str());
    }
    comments.retain(|c| {
        // Unwrap is safe since we checked above in the forloop
        let created_at = c["created_at"].as_str().unwrap().to_string();
        if created_at > issue.last_update {
            if created_at > new_value {
                new_value = created_at;
                update = true;
            }
            true
        } else {
            false
        }
    });
    if update {
        if issue.subscribed {
            let comments = comments
                .into_iter()
                .map(|c| {
                    let u = &c["user"];
                    Ok(
                        if gitea_api_mismatch!(u["login"].as_str()) == conf.bot_username {
                            gitea_api_mismatch!(c["body"].as_str()).to_string()
                        } else {
                            format!(
                                "User {} replied:\n\n{}",
                                c["user"]["login"],
                                gitea_api_mismatch!(c["body"].as_str())
                            )
                        },
                    )
                })
                .collect::<Result<Vec<String>>>()?;
            let mut notice = melib::Draft::default();
            notice.headers_mut().insert(
                HeaderName::new_unchecked("From"),
                Address::new(
                    None,
                    format!(
                        "{local_part}@{domain}",
                        local_part = &conf.local_part,
                        domain = &conf.domain
                    ),
                )
                .to_string(),
            );
            notice.headers_mut().insert(
                HeaderName::new_unchecked("Subject"),
                format!(
                    "[{tag}] new replies in issue `{title}`",
                    tag = &conf.tag,
                    title = &issue.title
                ),
            );
            notice
                .headers_mut()
                .insert(HeaderName::new_unchecked("To"), issue.submitter.to_string());

            notice.set_body(templates::reply_update(&issue, conf, comments));
            send_mail(notice, conf)?;
        }
        if !conf.dry_run {
            let mut stmt =
                conn.prepare("UPDATE issue SET last_update = (:last_update) WHERE id = (:id)")?;
            assert_eq!(
                stmt.execute(
                    rusqlite::named_params! {":last_update": &new_value, ":id": &issue.id}
                )?,
                1
            );
        }
    }

    Ok(update)
}

pub fn check(conn: Connection, conf: Configuration) -> Result<()> {
    let mut stmt = conn.prepare("SELECT * FROM issue")?;
    let results = stmt
        .query_map([], |row| {
            let submitter: String = row.get(1)?;
            let password: uuid::Uuid = row.get(2)?;
            let last_update: Option<String> = row.get(7)?;
            Ok(Issue {
                id: row.get(0)?,
                submitter: Address::new(None, submitter.as_str().to_string()),
                password,
                time_created: row.get(3)?,
                anonymous: row.get(4)?,
                subscribed: row.get(5)?,
                title: row.get(6)?,
                last_update: last_update.unwrap_or_default(),
            })
        })?
        .collect::<std::result::Result<Vec<Issue>, _>>()?;
    let mut errors: Vec<Result<bool>> = vec![];
    for issue in results {
        errors.push(check_issue(&conn, &conf, issue));
    }
    let successes_count = errors.iter().filter(|r| matches!(r, Ok(true))).count();
    let error_count = errors.iter().filter(|r| r.is_err()).count();
    log::info!(
        "Cron run with {} updates and {} errors.",
        successes_count,
        error_count
    );
    _ = errors.into_iter().collect::<Result<Vec<bool>>>()?;
    Ok(())
}
