use super::*;

static BASE_ISSUE_URL: &'static str = "{base_url}/{repo}/issues";

pub fn new_issue_failure(e: IssueError, conf: &Config) -> String {
    format!("Hello,

Unfortunately we were not able to create your issue. The reason was: `{}`. Please contact the repository's owners for assistance.

This is an automated email from {bot_name} <{local_part}+help@{domain}>",  e.to_string(), local_part = &conf.local_part, domain = &conf.domain, bot_name = &conf.bot_name)
}

pub fn new_issue_success(
    title: String,
    password: Password,
    issue_id: i64,
    conf: &Config,
) -> String {
    format!("Hello,

You have successfully submitted an issue titled \"{title}\". Your issue can be found at

{url}/{issue_id}

You will receive replies from other users. To unsubscribe from the conversation, send an email to {local_part}+{password}+unsubscribe@{domain}.

To reply to other users or post new comments, send your text to {local_part}+{password}+reply@{domain}.

To close the issue, send an email to {local_part}+{password}+close@{domain}.

Please keep this email in order to be able to keep in touch with your issue.

This is an automated email from {bot_name} <{local_part}+help@{domain}>", title = title, password = password.to_string(), issue_id = issue_id, url = BASE_ISSUE_URL.replace("{base_url}", &conf.base_url).replace("{repo}", &conf.repo), local_part = &conf.local_part, domain = &conf.domain, bot_name = &conf.bot_name)
}

pub fn new_reply_failure(e: IssueError, conf: &Config) -> String {
    format!("Hello,

Unfortunately we were not able to post your reply. The reason was: `{}`. Please contact the repository's owners for assistance.

This is an automated email from {bot_name} <{local_part}+help@{domain}>",  e.to_string(), local_part = &conf.local_part, domain = &conf.domain, bot_name = &conf.bot_name)
}

pub fn new_reply_success(
    title: String,
    password: Password,
    issue_id: i64,
    is_subscribed: bool,
    conf: &Config,
) -> String {
    if is_subscribed {
        format!("Hello,

Your reply to issue \"{title}\" has been successfully posted. You can view the discussion here:

{url}/{issue_id}

You will receive replies from other users. To unsubscribe from the conversation, send an email to {local_part}+{password}+unsubscribe@{domain}.

To reply to other users or post new comments, send your text to {local_part}+{password}+reply@{domain}.

To close the issue, send an email to {local_part}+{password}+close@{domain}.

Please keep this email in order to be able to keep in touch with your issue.

This is an automated email from {bot_name} <{local_part}+help@{domain}>", title = title, password = password.to_string(), issue_id = issue_id, url = BASE_ISSUE_URL.replace("{base_url}", &conf.base_url).replace("{repo}", &conf.repo), local_part = &conf.local_part, domain = &conf.domain, bot_name = &conf.bot_name)
    } else {
        format!("Hello,

Your reply to issue \"{title}\" has been successfully posted. You can view the discussion here:

{url}/{issue_id}

You will not receive replies from other users. To subscribe to the conversation, send an email to {local_part}+{password}+subscribe@{domain}.

To reply to other users or post new comments, send your text to {local_part}+{password}+reply@{domain}.

To close the issue, send an email to {local_part}+{password}+close@{domain}.

Please keep this email in order to be able to keep in touch with your issue.

This is an automated email from {bot_name} <{local_part}+help@{domain}>", title = title, password = password.to_string(), issue_id = issue_id, url = BASE_ISSUE_URL.replace("{base_url}", &conf.base_url).replace("{repo}", &conf.repo), local_part = &conf.local_part, domain = &conf.domain, bot_name = &conf.bot_name)
    }
}

pub fn close_success(title: String, issue_id: i64, conf: &Config) -> String {
    format!(
        "Hello,

Your issue \"{title}\" has been successfully closed. You can view the discussion here:

{url}/{issue_id}

This is an automated email from {bot_name} <{local_part}+help@{domain}>",
        title = title,
        issue_id = issue_id,
        url = BASE_ISSUE_URL
            .replace("{base_url}", &conf.base_url)
            .replace("{repo}", &conf.repo),
        local_part = &conf.local_part,
        domain = &conf.domain,
        bot_name = &conf.bot_name,
    )
}

pub fn close_failure(e: IssueError, conf: &Config) -> String {
    format!("Hello,

Unfortunately we were not able to close this issue. The reason was: `{}`. Please contact the repository's owners for assistance.

This is an automated email from {bot_name} <{local_part}+help@{domain}>",  e.to_string(), local_part = &conf.local_part, domain = &conf.domain, bot_name = &conf.bot_name)
}

pub fn invalid_request(conf: &Config) -> String {
    format!(
        "Hello,

Your request was not correct. Here are the valid requests you can ask from this bot:

- post a new issue eponymously: send an e-mail with the issue title as the subject and the issue body as the email body to {local_part}@{domain}. On success a password will be given that allows you to reply, close the issue, and also change your subscription to the discussion.
- post a new issue anonymously: send an email as above to the address {local_part}+anonymous@{domain}. {bot_name} will replace your name with 'Anonymous'

If p is the given password, you may perform actions on your issue as follows:

- reply: {local_part}+p+reply@{domain}. Subject value can be anything.
- close issue: {local_part}+p+close@{domain} email content can be anything
- change subscription: {local_part}+p+unsubscribe@{domain} and {local_part}+p+subscribe@{domain}

This is an automated email from {bot_name} <{local_part}+help@{domain}>",
        local_part = &conf.local_part,
        domain = &conf.domain,
        bot_name = &conf.bot_name,
    )
}

pub fn change_subscription_success(
    title: String,
    password: Password,
    issue_id: i64,
    is_subscribed: bool,
    conf: &Config,
) -> String {
    format!("Hello,

Your subscription change to issue \"{title}\" has been successfully performed. You can view the discussion here:

{url}/{issue_id}

You will {not}receive replies from other users. To {un}subscribe to the conversation, send an email to {local_part}+{password}+{un}subscribe@{domain}.

To reply to other users or post new comments, send your text to {local_part}+{password}+reply@{domain}.

To close the issue, send an email to {local_part}+{password}+close@{domain}.

Please keep this email in order to be able to keep in touch with your issue.

This is an automated email from {bot_name} <{local_part}+help@{domain}>", title = title, password = password.to_string(), issue_id = issue_id, url = BASE_ISSUE_URL.replace("{base_url}", &conf.base_url).replace("{repo}", &conf.repo), local_part = &conf.local_part, domain = &conf.domain, bot_name = &conf.bot_name, not = if is_subscribed { "" }else {"not "}, un = if is_subscribed { "un" } else { "" } )
}

pub fn change_subscription_failure(is_subscribed: bool, conf: &Config) -> String {
    format!(
        "Hello,

Your subscription change was unsuccessful. You are already {un}subscribed.

This is an automated email from {bot_name} <{local_part}+help@{domain}>",
        local_part = &conf.local_part,
        domain = &conf.domain,
        bot_name = &conf.bot_name,
        un = if is_subscribed { "un" } else { "" }
    )
}

pub fn reply_update(issue: &Issue, conf: &Config, comments: Vec<String>) -> String {
    assert!(comments.len() > 0);
    format!(
        "Hello,

There have been new replies in issue `{title}`. You are receiving this notice because you are subscribed to the discussion. To unsubscribe, send an email to {local_part}+{password}+unsubscribe@{domain}

{comments}

This is an automated email from {bot_name} <{local_part}+help@{domain}>",
        local_part = &conf.local_part,
        domain = &conf.domain,
        bot_name = &conf.bot_name,
        password = &issue.password.to_string(),
        title = &issue.title,
        comments = comments.join("\n\n-------------------------------------------------------------------------\n\n")
    )
}
