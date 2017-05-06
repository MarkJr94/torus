use std::io;
use std::num::ParseIntError;
use rusqlite;
use xdg;

error_chain! {
    foreign_links {
        Io(io::Error);
        Db(rusqlite::Error);
        ParseInt(ParseIntError);
        ParseStr(::std::string::ParseError);
        Xdg(xdg::BaseDirectoriesError);
    }

    errors {
        NoSuchEntry(id: u32) {
            description("Invalid entry id")
            display("No entry with id `{}`", id)
        }
        MissingArg(arg: &'static str) {
            description("Missing argument")
            display("Required argument `{}` is missing", arg)
        }
        BadRating {
            description("Ratings must be integers between 1 and 5")
            display("Rating is not an integer between 1 and 5")
        }
    }
}