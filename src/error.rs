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

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
        Database(rusqlite::Error);
        Unicode(std::str::Utf8Error);
        UnicodeS(std::string::FromUtf8Error);
        Email(melib::error::MeliError);
   }
}

impl Error {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        msg.into().into()
    }
}
