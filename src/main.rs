#[macro_use]
extern crate prettytable;
extern crate clap;
extern crate rustyline;
extern crate xdg;
extern crate rusqlite;

use std::error::Error;
use std::io::{self, Write, stdout};
use std::path::Path;

use clap::{Arg, App, SubCommand};

use prettytable::Table;
use prettytable::row::Row;

use rusqlite::Connection;

use xdg::BaseDirectories;

const NAME: &'static str = "torus";
const DBNAME: &'static str = "torus.db";

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    id: i32,
    name: String,
    read: bool,
    page: u32,
    genre: String,
    author: String,
}

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
        Some(path) => {
            println!("found db");
            Connection::open(path).expect("Couldn't open db")
        }
        None => {
            println!("Creating db");
            let p = bd.place_data_file(DBNAME).expect("Couldn't get db");
            create_db(&p).expect("Couldn't create db")
        }
    };

    p
}

fn do_add(conn: &Connection, entry: &Entry) -> Result<i32, rusqlite::Error> {
    conn.execute("INSERT INTO entry (name, author, read, page, genre)
                  VALUES (?1, ?2, ?3, ?4, ?5)",
                 &[&entry.name,
                   &entry.author,
                   &entry.read,
                   &entry.page,
                   &entry.genre])
}

fn do_list(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, name, author, page, genre, read FROM entry
                                 ORDER BY page DESC")?;

    let entries = stmt.query_map(&[], |row| {
            Entry {
                id: row.get(0),
                name: row.get(1),
                author: row.get(2),
                page: row.get(3),
                genre: row.get(4),
                read: row.get(5),
            }
        })?;


    let mut table = Table::new();
    table.add_row(row!["ID", "Name", "AUTHOR", "GENRE", "PAGE", "READ"]);

    for entry in entries {
        let entry = entry?;

        let row = row![&entry.id.to_string(),
                       &entry.name.to_string(),
                       &entry.author.to_string(),
                       &entry.genre.to_string(),
                       &entry.page.to_string(),
                       &entry.read.to_string()];

        table.add_row(row);
    }

    table.printstd();

    Ok(())
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
        .subcommand(SubCommand::with_name("list").about("list entries in order of page"));

    let matches = app.get_matches();

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


        do_add(&conn, &entry).expect("Failed to add entry :(");
        println!("Successfully added {} by {}", entry.name, entry.author);
    }

    if let Some(_) = matches.subcommand_matches("list") {
        do_list(&conn).expect("unable to list entries for some reason :(");
    }


    println!("Hello, world!");
}
