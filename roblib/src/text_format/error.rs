use serde::{de, ser};
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),
    MissingArgument,
    FormatterError(fmt::Error),
    UnsizedSeq,
    UnsizedMap,
    DeserializeAny,
    Parse(&'static str),
    Trailing,
    Empty,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::MissingArgument => formatter.write_str("missing argument"),
            Error::FormatterError(e) => formatter.write_str(&e.to_string()),
            Error::UnsizedSeq => formatter.write_str("sequence length not specified"),
            Error::UnsizedMap => formatter.write_str("map length not specified"),
            Error::DeserializeAny => formatter.write_str("can't deserialize arbitrary data"),
            Error::Parse(ty) => formatter.write_fmt(format_args!("failed to parse {ty}")),
            Error::Trailing => formatter.write_str("found trailing characters"),
            Error::Empty => formatter.write_str("empty argument"),
        }
    }
}

impl std::error::Error for Error {}

impl From<fmt::Error> for Error {
    fn from(value: fmt::Error) -> Self {
        Self::FormatterError(value)
    }
}
