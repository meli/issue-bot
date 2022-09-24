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

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    /// eg. meli-issues becomes [meli-issues]
    pub tag: String,
    /// your bot's authentication token from Gitea's Swagger
    pub auth_token: String,
    /// eg. for issues@meli.delivery the local part is issues
    pub local_part: String,
    /// eg. for issues@meli.delivery the domain is meli.delivery
    pub domain: String,
    /// eg. "https://git.meli.delivery"
    pub base_url: String,
    /// eg. "meli/meli"
    pub repo: String,
    /// The bot's name that will be displayed in signatures of sent replies
    pub bot_name: String,
    /// The bot's login username
    pub bot_username: String,
    /// the command to pipe an email to
    pub mailer: String,
    /// file to write logs
    pub log_file: String,
    /// don't actually email anything
    #[serde(default)]
    pub dry_run: bool,
}
