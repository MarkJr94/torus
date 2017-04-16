#[macro_use]
extern crate prettytable;
extern crate clap;
extern crate linefeed;
extern crate xdg;
extern crate rusqlite;
extern crate cmdline_parser;

mod commands;
mod data;
mod err;

use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{Arg, App, SubCommand};

use cmdline_parser::Parser;

use rusqlite::Connection;

use linefeed::{Reader, ReadResult};

use xdg::BaseDirectories;

use data::Entry;
use err::TErr;
use commands::{Command, exec_command};

const NAME: &'static str = "torus";
const DBNAME: &'static str = "torus.db";
const HIST_NAME: &'static str = "history.txt";
const PROMPT: &'static str = "torus->> ";

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

fn init() -> (Connection, PathBuf) {
    let bd = get_base().expect("Couldn't get base dirs");

    let db_path = bd.find_data_file(DBNAME);
    let hist_path = bd.find_data_file(HIST_NAME);

    let p = match db_path {
        Some(path) => Connection::open(path).expect("Couldn't open db"),
        None => {
            println!("Creating Database");
            let p = bd.place_data_file(DBNAME)
                .expect("Couldn't open new Database");
            create_db(&p).expect("Couldn't create db")
        }
    };

    let hist_path = match hist_path {
        Some(p) => p,
        None => {
            println!("Initializing history");
            bd.place_data_file(HIST_NAME)
                .expect("Couldn't initalize history")
        }
    };

    (p, hist_path)
}

fn ep<'a>(conn: &Connection, matches: clap::ArgMatches<'a>) -> Result<(), err::TErr> {
    let mut command = Command::Nil;

    if let Some(add) = matches.subcommand_matches("add") {
        let entry = Entry {
            name: add.value_of("TITLE")
                .ok_or(TErr::MissingArg("TITLE"))?
                .into(),
            genre: add.value_of("GENRE")
                .ok_or(TErr::MissingArg("GENRE"))?
                .into(),
            page: add.value_of("PAGE")
                .ok_or(TErr::MissingArg("PAGE"))?
                .parse()?,
            author: add.value_of("AUTHOR")
                .ok_or(TErr::MissingArg("AUTHOR"))?
                .into(),
            read: false,
            id: 0,
        };

        command = Command::Add(entry);
    }

    if let Some(_) = matches.subcommand_matches("list") {
        command = Command::List;
    }

    if let Some(search) = matches.subcommand_matches("search") {
        let query = search
            .value_of("QUERY")
            .ok_or(TErr::MissingArg("QUERY"))?
            .into();

        command = Command::Search(query);
    }

    if let Some(_) = matches.subcommand_matches("choose") {
        command = Command::Choose;
    }

    if let Some(finish) = matches.subcommand_matches("finish") {
        let id = finish
            .value_of("ENTRY_ID")
            .ok_or(TErr::MissingArg("ENTRY_ID"))?
            .parse()?;

        command = Command::Finish(id);
    }

    if let Some(delete) = matches.subcommand_matches("delete") {
        let id = delete
            .value_of("ENTRY_ID")
            .ok_or(TErr::MissingArg("ENTRY_ID"))?
            .parse()?;

        command = Command::Delete(id);
    }

    if let Some(set_page) = matches.subcommand_matches("set-page") {
        let id = set_page
            .value_of("ENTRY_ID")
            .ok_or(TErr::MissingArg("ENTRY_ID"))?
            .parse()?;

        let page = set_page
            .value_of("PAGE")
            .ok_or(TErr::MissingArg("PAGE"))?
            .parse()?;

        command = Command::SetPage(id, page);
    }

    let msg_res = exec_command(&conn, command.clone());

    let msg =
        msg_res.unwrap_or_else(|err| format!("Command {:?} failed. Caused by: {}", command, err));

    println!("{}", msg);

    Ok(())
}
fn dump_history<C: linefeed::Terminal>(reader: &Reader<C>,
                                       path: &PathBuf)
                                       -> Result<(), io::Error> {
    use std::fs::File;

    let mut file = File::create(path)?;

    for line in reader.history() {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

fn load_history<C: linefeed::Terminal>(reader: &mut Reader<C>,
                                       path: &PathBuf)
                                       -> Result<(), io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path)?;
    let f = BufReader::new(file);

    for line in f.lines() {
        let line = line?;
        reader.add_history(line);
    }

    Ok(())
}


fn main() {
    let (conn, hist_path) = init();

    let app = App::new(NAME)
        .author("Mark <mark.edward.x@gmail.com>")
        .about("CLI Reading List application")
        .version("0.1")
        //.setting(clap::AppSettings::DisableHelpSubcommand)
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
                         .help("The last page you read of this entry")))
        .subcommand(SubCommand::with_name("shell")
                    .about("Enter interactive mode"));

    let matches = app.clone().get_matches();

    if let Some(_) = matches.subcommand_matches("shell") {
        let app = app.setting(clap::AppSettings::NoBinaryName);

        let mut reader = Reader::new("torus").expect("Couldn't open linereader");

        load_history(&mut reader, &hist_path).unwrap_or_else(|e| {
            writeln!(io::stderr(), "Couldn't load history due to {}", e.description())
                .unwrap();
        });

        reader.set_prompt(PROMPT);

        while let Ok(ReadResult::Input(input)) = reader.read_line() {
            reader.add_history(input.clone());
            let parser = Parser::new(&input);

            let args_it = parser.map(|(_, s)| s);

            let matches = app.clone().get_matches_from_safe(args_it);

            let _ = matches
                .map(|matches| {
                         ep(&conn, matches).unwrap_or_else(|e| {
                                                               writeln!(io::stderr(),
                                                                        "Error: {}",
                                                                        e)
                                                                       .unwrap();
                                                           });
                     })
                .map_err(|err| if err.kind != clap::ErrorKind::HelpDisplayed ||
                                  err.kind != clap::ErrorKind::VersionDisplayed {
                             writeln!(io::stderr(), "{}", err).unwrap();
                         })
                .unwrap_or(());
        }

        dump_history(&reader, &hist_path).unwrap_or_else(|e| {
                                                             writeln!(io::stderr(),
                                                                      "Failed to dump history: {}",
                                                                      e)
                                                                     .unwrap();
                                                         });

    } else {

        ep(&conn, matches).unwrap_or_else(|e| { writeln!(io::stderr(), "Error: {}", e).unwrap(); });
    }
}
