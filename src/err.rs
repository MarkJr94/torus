use std::error;
use std::fmt;

use rusqlite;

#[derive(Debug)]
pub enum TErr {
    Db(rusqlite::Error),
    NoSuchEntry(u32),
}

impl From<rusqlite::Error> for TErr {
    fn from(err: rusqlite::Error) -> TErr {
        TErr::Db(err)
    }
}

impl fmt::Display for TErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TErr::Db(ref err) => write!(f, "SQLite Error: {}", err),
            TErr::NoSuchEntry(id) => write!(f, "No entry with id {}", id),
        }
    }
}

impl error::Error for TErr {
    fn description(&self) -> &str {
        match *self {
            TErr::Db(ref err) => err.description(),
            TErr::NoSuchEntry(_) => "Invalid entry id",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            TErr::Db(ref err) => Some(err),
            TErr::NoSuchEntry(_) => None,
        }
    }
}
