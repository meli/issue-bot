use super::*;

static ISSUES_BASE_URL: &'static str = "{base_url}/api/v1/repos/{repo}/issues";
static ISSUES_COMMENTS_URL: &'static str = "{base_url}/api/v1/repos/{repo}/issues/{index}/comments";

use serde::Serialize;

#[derive(Serialize, Default)]
struct CreateIssueOption {
    assignee: String,
    assignees: Vec<String>,
    body: String,
    closed: bool,
    title: String,
}

pub fn new_issue(
    conn: &Connection,
    title: String,
    body: String,
    anonymous: bool,
    submitter: Address,
    conf: &Config,
) -> Result<(Password, i64)> {
    let issue = CreateIssueOption {
        title,
        body: format!(
            "{} reports:\n\n{}",
            if anonymous {
                "Anonymous".to_string()
            } else {
                submitter.to_string()
            },
            body
        ),
        ..CreateIssueOption::default()
    };
    let client = reqwest::Client::new();
    let res = client
        .post(
            &ISSUES_BASE_URL
                .replace("{base_url}", &conf.base_url)
                .replace("{repo}", &conf.repo),
        )
        .header("Authorization", format!("token {}", &conf.auth_token))
        .json(&issue)
        .send()
        .unwrap()
        .text()
        .unwrap();

    let map: serde_json::map::Map<String, serde_json::Value> = serde_json::from_str(&res).unwrap();
    let issue = Issue {
        id: map["number"].as_i64().unwrap(),
        submitter,
        password: Uuid::new_v4(),
        time_created: time::get_time(),
        anonymous,
        subscribed: true,
        title: issue.title,
        last_update: map["created_at"].to_string(),
    };
    conn.execute(
        "INSERT INTO issue (id, submitter, password, time_created, anonymous, subscribed, title, last_update)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        &[
            &issue.id,
            &issue.submitter.to_string() as &dyn ToSql,
            &issue.password.as_bytes().to_vec(),
            &issue.time_created,
            &issue.anonymous,
            &issue.subscribed,
            &issue.title,
            &issue.last_update,
        ],
    )
    .unwrap();
    Ok((issue.password, issue.id))
}

#[derive(Serialize, Default)]
struct CreateIssueCommentOption {
    body: String,
}

pub fn new_reply(
    conn: &Connection,
    body: String,
    password: Password,
    submitter: Address,
    conf: &Config,
) -> Result<(String, i64, bool)> {
    let mut stmt =
        conn.prepare("SELECT id, title, subscribed, anonymous FROM issue WHERE password = ?")?;
    let mut results = stmt
        .query_map(&[password.as_bytes().to_vec()], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .map(|r| r.unwrap())
        .collect::<Vec<(i64, String, bool, bool)>>();
    if results.is_empty() {
        return Err(IssueError::new("Not found".to_string()));
    }
    let client = reqwest::Client::new();
    let response = client
        .post(
            &ISSUES_COMMENTS_URL
                .replace("{base_url}", &conf.base_url)
                .replace("{repo}", &conf.repo)
                .replace("{index}", &results[0].0.to_string()),
        )
        .header("Authorization", format!("token {}", &conf.auth_token))
        .json(&CreateIssueCommentOption {
            body: format!(
                "{} replies:\n\n{}",
                if results[0].3 {
                    "Anonymous".to_string()
                } else {
                    submitter.to_string()
                },
                body
            ),
        })
        .send()?;
    if response.status().is_success() {
        let (issue_id, title, is_subscribed, _) = results.remove(0);
        Ok((title, issue_id, is_subscribed))
    } else {
        eprintln!(
            "New reply could not be created: {:?}\npassword: {}\nsubmitter: {}\nbody: {}",
            response.status(),
            password.to_string(),
            submitter.to_string(),
            body
        );
        Err(IssueError::new(
            "You can not reply to this issue due to an internal error.",
        ))
    }
}

#[derive(Serialize, Default)]
struct EditIssueOption {
    state: String,
}

pub fn close(conn: &Connection, password: Password, conf: &Config) -> Result<(String, i64, bool)> {
    let mut stmt = conn.prepare("SELECT id, title, subscribed FROM issue WHERE password = ?")?;
    let mut results = stmt
        .query_map(&[password.as_bytes().to_vec()], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .map(|r| r.unwrap())
        .collect::<Vec<(i64, String, bool)>>();
    if results.is_empty() {
        return Err(IssueError::new("Not found".to_string()));
    }
    let client = reqwest::Client::new();
    let res = client
        .patch(&format!(
            "{}/{}",
            ISSUES_BASE_URL
                .replace("{base_url}", &conf.base_url)
                .replace("{repo}", &conf.repo),
            &results[0].0.to_string()
        ))
        .header("Authorization", format!("token {}", &conf.auth_token))
        .json(&EditIssueOption {
            state: "closed".to_string(),
        })
        .send()?
        .text()?;

    let map: serde_json::map::Map<String, serde_json::Value> = serde_json::from_str(&res).unwrap();
    if map["state"] == "closed" {
        let (issue_id, title, is_subscribed) = results.remove(0);
        Ok((title, issue_id, is_subscribed))
    } else {
        eprintln!("Issue could not be closed: {:#?}", map);
        Err(IssueError::new(
            "Issue cannot be closed due to an internal error.",
        ))
    }
}

pub fn change_subscription(
    conn: &Connection,
    password: Password,
    new_val: bool,
) -> Result<(String, i64, bool)> {
    let mut stmt = conn.prepare("SELECT id, title, subscribed FROM issue WHERE password = ?")?;
    let mut results = stmt
        .query_map(&[password.as_bytes().to_vec()], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .map(|r| r.unwrap())
        .collect::<Vec<(i64, String, bool)>>();
    if results.is_empty() {
        return Err(IssueError::new("Issue not found".to_string()));
    }
    let (issue_id, title, is_subscribed) = results.remove(0);
    if !is_subscribed && !new_val {
        return Err(IssueError::new(format!(
            "You are not subscribed to issue `{}`",
            &title
        )));
    } else if is_subscribed && new_val {
        return Err(IssueError::new(format!(
            "You are already subscribed to issue `{}`",
            &title
        )));
    }

    let mut stmt =
        conn.prepare("UPDATE issue SET subscribed = (:subscribed) WHERE password = (:password)")?;
    assert_eq!(
        stmt.execute_named(&[
            (":subscribed", &new_val),
            (":password", &password.as_bytes().to_vec())
        ])?,
        1
    );
    Ok((title, issue_id, is_subscribed))
}

pub fn comments(
    id: i64,
    since: &str,
    conf: &Config,
) -> Vec<serde_json::map::Map<String, serde_json::Value>> {
    let client = reqwest::Client::new();
    let result = client
        .get(
            &ISSUES_COMMENTS_URL
                .replace("{base_url}", &conf.base_url)
                .replace("{repo}", &conf.repo)
                .replace("{index}", &id.to_string()),
        )
        .header("Authorization", format!("token {}", &conf.auth_token))
        .query(&[("since", since)])
        .send()
        .unwrap()
        .text()
        .unwrap();
    let result: Vec<_> = serde_json::from_str(&result).unwrap();
    result
}
