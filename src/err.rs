use std::error;
use std::fmt;
use std::num::ParseIntError;

use rusqlite;

#[derive(Debug)]
pub enum TErr {
    Db(rusqlite::Error),
    NoSuchEntry(u32),
    Parse(ParseIntError),
    MissingArg(&'static str),
    BadRating,
}

impl From<rusqlite::Error> for TErr {
    fn from(err: rusqlite::Error) -> TErr {
        TErr::Db(err)
    }
}

impl From<ParseIntError> for TErr {
    fn from(err: ParseIntError) -> TErr {
        TErr::Parse(err)
    }
}

impl fmt::Display for TErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TErr::Db(ref err) => write!(f, "SQLite Error: {}", err),
            TErr::NoSuchEntry(id) => write!(f, "No entry with id {}", id),
            TErr::Parse(ref err) => write!(f, "Parsing error: {}", err),
            TErr::MissingArg(arg) => write!(f, "Missing argument <{}>", arg),
            TErr::BadRating => write!(f, "Ratings must be between 1 and 5"),
        }
    }
}

impl error::Error for TErr {
    fn description(&self) -> &str {
        match *self {
            TErr::Db(ref err) => err.description(),
            TErr::NoSuchEntry(_) => "Invalid entry id",
            TErr::Parse(_) => "Parsing error",
            TErr::MissingArg(_) => "Missing argument",
            TErr::BadRating => "Ratings must be between 1 and 5",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            TErr::Db(ref err) => Some(err),
            TErr::NoSuchEntry(_) => None,
            TErr::Parse(ref err) => Some(err),
            TErr::MissingArg(_) => None,
            TErr::BadRating => None,
        }
    }
}
