
#[macro_use]
extern crate prettytable;
extern crate clap;
extern crate rustyline;
extern crate xdg;
extern crate rusqlite;

mod commands;
mod data;
mod err;

use std::error::Error;
use std::io::{self, Write, stdout};
use std::path::Path;

use clap::{Arg, App, SubCommand};

use rusqlite::Connection;

use xdg::BaseDirectories;

use err::TErr;
use data::Entry;
use commands::{Command, exec_command};

const NAME: &'static str = "torus";
const DBNAME: &'static str = "torus.db";

fn create_db<T: AsRef<Path>>(path: &T) -> Result<Connection, Box<Error>> {
    let conn = Connection::open(path)?;

    conn.execute("CREATE TABLE entry (
                 id INTEGER PRIMARY KEY,
                 author TEXT NOT NULL,
                 name TEXT NOT NULL,
                 read BOOLEAN NOT NULL,
                 page INTEGER NOT NULL,
                 genre TEXT NOT NULL)",
                 &[])?;

    Ok(conn)
}

fn get_base() -> Result<BaseDirectories, Box<Error>> {
    let bd = BaseDirectories::with_prefix(NAME)?;

    Ok(bd)
}

fn init() -> Connection {
    let bd = get_base().expect("Couldn't get base dirs");

    let db_path = bd.find_data_file(DBNAME);

    let p = match db_path {
        Some(path) => Connection::open(path).expect("Couldn't open db"),
        None => {
            println!("Creating Database");
            let p = bd.place_data_file(DBNAME)
                .expect("Couldn't open new Database");
            create_db(&p).expect("Couldn't create db")
        }
    };

    p
}


fn main() {
    let conn = init();

    let app = App::new(NAME)
        .author("Mark <mark.edward.x@gmail.com>")
        .about("reading list")
        .version("0.1")
        .subcommand(SubCommand::with_name("add")
                        .about("add entry")
                        .arg(Arg::with_name("TITLE")
                                 .required(true)
                                 .short("t")
                                 .index(1)
                                 .help("title of entry"))
                        .arg(Arg::with_name("AUTHOR")
                                 .required(true)
                                 .index(2)
                                 .help("Author of work"))
                        .arg(Arg::with_name("GENRE")
                                 .required(true)
                                 .index(3)
                                 .help("Genre of work"))
                        .arg(Arg::with_name("PAGE")
                                 .default_value("0")
                                 .help("Page you are currently at")))
        .subcommand(SubCommand::with_name("list").about("list entries in order of page"))
        .subcommand(SubCommand::with_name("search")
                    .about("find entries. case insensitive match on 'TITLE', 'AUTHOR', and 'GENRE'")
                    .arg(Arg::with_name("QUERY")
                         .required(true)
                         .index(1)
                         .help("search query")))
        .subcommand(SubCommand::with_name("choose")
                    .about("Choose a random entry for you to read"))
        .subcommand(SubCommand::with_name("finish")
                    .about("Mark an entry as read")
                    .arg(Arg::with_name("ENTRY_ID")
                         .required(true)
                         .index(1)
                         .help("ID of entry to mark as read (acquire from `search` or `list`)")))
        .subcommand(SubCommand::with_name("delete")
                    .about("Delete an entry")
                    .arg(Arg::with_name("ENTRY_ID")
                         .required(true)
                         .index(1)
                         .help("ID of entry to delete (acquire from `search` or `list`)")))
        .subcommand(SubCommand::with_name("set-page")
                    .about("Set the last page you read for an entry")
                    .arg(Arg::with_name("ENTRY_ID")
                         .required(true)
                         .index(1)
                         .help("ID of entry to modify (acquire from `search` or `list`)"))
                    .arg(Arg::with_name("PAGE")
                         .required(true)
                         .index(2)
                         .help("The last page you read of this entry")));

    let matches = app.get_matches();


    let mut command = Command::Nil;
    if let Some(add) = matches.subcommand_matches("add") {
        let entry = Entry {
            name: add.value_of("TITLE").unwrap().into(),
            genre: add.value_of("GENRE").unwrap().into(),
            page: add.value_of("PAGE")
                .unwrap()
                .parse()
                .expect("Invalid number for page"),
            author: add.value_of("AUTHOR").unwrap().into(),
            read: false,
            id: 0,
        };

        command = Command::Add(entry);
    }

    if let Some(_) = matches.subcommand_matches("list") {
        command = Command::List;
    }

    if let Some(search) = matches.subcommand_matches("search") {
        let query = search.value_of("QUERY").unwrap().into();

        command = Command::Search(query);
    }

    if let Some(_) = matches.subcommand_matches("choose") {
        command = Command::Choose;
    }

    if let Some(finish) = matches.subcommand_matches("finish") {
        let id = finish
            .value_of("ENTRY_ID")
            .unwrap()
            .parse()
            .expect("ID must be positive integer");

        command = Command::Finish(id);
    }

    if let Some(delete) = matches.subcommand_matches("delete") {
        let id = delete
            .value_of("ENTRY_ID")
            .unwrap()
            .parse()
            .expect("ID must be positive integer");

        command = Command::Delete(id);
    }

    if let Some(set_page) = matches.subcommand_matches("set-page") {
        let id = set_page
            .value_of("ENTRY_ID")
            .unwrap()
            .parse()
            .expect("ID must be positive integer");

        let page = set_page
            .value_of("PAGE")
            .unwrap()
            .parse()
            .expect("PAGE must be a positive integer");

        command = Command::SetPage(id, page);
    }

    let msg = exec_command(&conn, command).unwrap();

    println!("{}", msg);
}
