use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    /** eg. meli-issues becomes [meli-issues] **/
    pub tag: String,
    /** your bot's authentication token from Gitea's Swagger **/
    pub auth_token: String,
    /** eg. for issues@meli.delivery the local part is issues **/
    pub local_part: String,
    /** eg. for issues@meli.delivery the domain is meli.delivery **/
    pub domain: String,
    /** eg. "https://git.meli.delivery" **/
    pub base_url: String,
    /** eg. "meli/meli" **/
    pub repo: String,
    /** The bot's name that will be displayed in signatures of sent replies **/
    pub bot_name: String,
    /** The bot's login username **/
    pub bot_username: String,
    /** the command to pipe an email to **/
    pub mailer: String,
}
