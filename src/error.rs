use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::io;
use std::result;
use std::str;
use std::string;

pub type Result<T> = result::Result<T, IssueError>;

#[derive(Debug, Clone)]
pub struct IssueError {
    details: String,
}

impl IssueError {
    pub fn new<M>(msg: M) -> IssueError
    where
        M: Into<String>,
    {
        IssueError {
            details: msg.into(),
        }
    }
}

impl fmt::Display for IssueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Into<String> for IssueError {
    fn into(self) -> String {
        self.details
    }
}

impl Error for IssueError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<io::Error> for IssueError {
    #[inline]
    fn from(kind: io::Error) -> IssueError {
        IssueError::new(kind.description())
    }
}

impl Into<io::Error> for IssueError {
    #[inline]
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.description())
    }
}

impl<'a> From<Cow<'a, str>> for IssueError {
    #[inline]
    fn from(kind: Cow<'_, str>) -> IssueError {
        IssueError::new(format!("{:?}", kind))
    }
}

impl From<string::FromUtf8Error> for IssueError {
    #[inline]
    fn from(kind: string::FromUtf8Error) -> IssueError {
        IssueError::new(format!("{:?}", kind))
    }
}

impl From<str::Utf8Error> for IssueError {
    #[inline]
    fn from(kind: str::Utf8Error) -> IssueError {
        IssueError::new(format!("{:?}", kind))
    }
}

impl From<rusqlite::Error> for IssueError {
    #[inline]
    fn from(kind: rusqlite::Error) -> IssueError {
        IssueError::new(format!("{}", kind.to_string()))
    }
}

impl From<reqwest::Error> for IssueError {
    #[inline]
    fn from(kind: reqwest::Error) -> IssueError {
        IssueError::new(format!("{}", kind.to_string()))
    }
}
