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

pub fn check(conn: Connection, conf: Configuration) -> Result<()> {
    let mut stmt = conn.prepare("SELECT * FROM issue")?;
    let mut results = stmt
        .query_map(NO_PARAMS, |row| {
            let submitter: String = row.get(1)?;
            let password: Vec<u8> = row.get(2)?;
            let last_update: Option<String> = row.get(7)?;
            Ok(Issue {
                id: row.get(0)?,
                submitter: Address::new(None, submitter.as_str().to_string()),
                password: Password::from_slice(password.as_slice()).unwrap(),
                time_created: row.get(3)?,
                anonymous: row.get(4)?,
                subscribed: row.get(5)?,
                title: row.get(6)?,
                last_update: last_update.unwrap_or(String::new()),
            })
        })?
        .collect::<std::result::Result<Vec<Issue>, _>>()?;
    for issue in &mut results {
        let mut update = false;
        let mut comments = api::comments(issue.id, &issue.last_update, &conf);
        let mut new_value = issue.last_update.clone();
        comments.retain(|c| {
            let created_at = c["created_at"].to_string();
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
            let mut stmt =
                conn.prepare("UPDATE issue SET last_update = (:last_update) WHERE id = (:id)")?;
            assert_eq!(
                stmt.execute_named(&[(":last_update", &new_value), (":id", &issue.id),])?,
                1
            );
            if issue.subscribed {
                let comments = comments
                    .into_iter()
                    .map(|c| {
                        if c["user"]["login"].as_str().unwrap() == &conf.bot_username {
                            c["body"].as_str().unwrap().to_string()
                        } else {
                            format!(
                                "User {} replied:\n\n{}",
                                c["user"]["login"],
                                c["body"].as_str().unwrap()
                            )
                        }
                    })
                    .collect::<Vec<String>>();
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
                    )
                    .to_string(),
                );
                notice
                    .headers_mut()
                    .insert(HeaderName::new_unchecked("To"), issue.submitter.to_string());

                notice.set_body(templates::reply_update(&issue, &conf, comments));
                send_mail(notice, &conf);
            }
        }
    }
    Ok(())
}
