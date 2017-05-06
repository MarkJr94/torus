use std::fmt;

use prettytable::Table;
use prettytable::cell::Cell;

use rusqlite::{self, Connection};

use time::{at, strftime, get_time};

use data::Entry;
use errors::*;
//use err::TErr;

const DATE_FMT: &'static str = "%D %T";

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    Add(Entry),
    Search(String),
    List,
    Finish(u32),
    Delete(u32),
    SetPage(u32, u32),
    Rate(u32, u8),
    Choose,
    Nil,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Command::Add(ref e) => write!(f, "Add' {}' by '{}'", e.name, e.author),
            Command::Search(ref query) => write!(f, "Search with query '{}'", query),
            Command::List => write!(f, "List entries"),
            Command::Finish(ref id) => write!(f, "Finish entry #{}", id),
            Command::Delete(ref id) => write!(f, "Delete entry #{}", id),
            Command::SetPage(ref id, ref pageno) => write!(f, "Set last-page-read to {}\
                                                       for entry #{}", pageno, id),
            Command::Rate(ref id, ref rating) => write!(f, "Rate entry #{} {} stars", id, rating),
            Command::Choose => write!(f, "Choose a random entry"),
            Command::Nil => write!(f, "Nil command for implementation reasons")
        }
    }
}

pub fn exec_command(conn: &Connection, command: Command) -> Result<String> {
    match command {
        Command::Add(ref entry) => do_add(conn, entry),
        Command::List => do_list(conn),
        Command::Search(ref term) => do_search(conn, term),
        Command::Finish(id) => do_finish(conn, id),
        Command::Delete(id) => do_delete(conn, id),
        Command::SetPage(id, page) => do_set_page(conn, id, page),
        Command::Rate(id, rating) => do_rate(conn, id, rating),
        Command::Choose => do_choose(conn),
        Command::Nil => Ok(String::new()),
    }
}

fn do_add(conn: &Connection, entry: &Entry) -> Result<String> {
    let _ = conn.execute("INSERT INTO entry (name, author, read, page, genre, date_added)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                         &[&entry.name,
                             &entry.author,
                             &entry.read,
                             &entry.page,
                             &entry.genre,
                             &entry.date_added])?;

    Ok(format!("Successfully added {} by {}", entry.name, entry.author))
}

fn do_list(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare("SELECT id, name, author, page, genre, read, date_added,
                                 date_finished, rating FROM entry
                                 ORDER BY page DESC")?;

    let entries = stmt.query_map(&[], |row| {
        Entry {
            id: row.get(0),
            name: row.get(1),
            author: row.get(2),
            page: row.get(3),
            genre: row.get(4),
            read: row.get(5),
            date_added: row.get(6),
            date_finished: row.get(7),
            rating: row.get(8),
        }
    })?;

    let _ = print_entries(entries)?;

    Ok("End of List".into())
}

fn do_search(conn: &Connection, term: &str) -> Result<String> {
    let mut stmt = conn.prepare("SELECT id, name, author, page, genre, read, \
                                 date_added, date_finished, rating FROM entry \
                                 WHERE name LIKE ?1 OR author LIKE ?1 or genre LIKE ?1 \
                                 ORDER BY page DESC")?;

    let query = format!("%{}%", term);

    let entries = stmt.query_map(&[&query], |row| {
        Entry {
            id: row.get(0),
            name: row.get(1),
            author: row.get(2),
            page: row.get(3),
            genre: row.get(4),
            read: row.get(5),
            date_added: row.get(6),
            date_finished: row.get(7),
            rating: row.get(8)
        }
    })?;

    let n = print_entries(entries)?;

    Ok(format!("Found {} result(s)", n))
}

fn do_finish(conn: &Connection, id: u32) -> Result<String> {
    let mut stmt = conn.prepare("UPDATE entry SET read = 1, date_finished = ?2 WHERE id = ?1")?;

    let time = get_time();

    let num_updated = stmt.execute(&[&id, &time])?;

    let good = num_updated > 0;

    if good {
        Ok(format!("Entry {} marked as read", id))
    } else {
        Err(ErrorKind::NoSuchEntry(id).into())
    }
}

fn do_rate(conn: &Connection, id: u32, rating: u8) -> Result<String> {
    let mut stmt = conn.prepare("UPDATE entry SET rating = ?2 WHERE id = ?1")?;

    if rating < 1 || rating > 5 {
        return Err(ErrorKind::BadRating.into());
    }

    let num_updated = stmt.execute(&[&id, &rating])?;

    let good = num_updated > 0;

    if good {
        Ok(format!("Entry {} rated {} stars", id, rating))
    } else {
        Err(ErrorKind::NoSuchEntry(id).into())
    }
}

fn do_delete(conn: &Connection, id: u32) -> Result<String> {
    let mut stmt = conn.prepare("DELETE FROM entry WHERE id = ?1")?;

    let num_deleted = stmt.execute(&[&id])?;

    if num_deleted > 0 {
        Ok(format!("Entry {} deleted", id))
    } else {
        Err(ErrorKind::NoSuchEntry(id).into())
    }
}

fn do_set_page(conn: &Connection, id: u32, page: u32) -> Result<String> {
    let mut stmt = conn.prepare("UPDATE entry SET page = ?2 where id = ?1")?;

    let num_updated = stmt.execute(&[&id, &page])?;

    if num_updated > 0 {
        Ok(format!("Set page to {} for entry {}", page, id))
    } else {
        Err(ErrorKind::NoSuchEntry(id).into())
    }
}

fn do_choose(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare("SELECT id, name, author, page, genre, read \
                                 date_added, date_finished, rating FROM entry \
                                 WHERE read = 0 \
                                 ORDER BY RANDOM() \
                                 LIMIT 1")?;

    let entries = stmt.query_map(&[], |row| {
        Entry {
            id: row.get(0),
            name: row.get(1),
            author: row.get(2),
            page: row.get(3),
            genre: row.get(4),
            read: row.get(5),
            date_added: row.get(6),
            date_finished: row.get(7),
            rating: row.get(8)
        }
    })?;

    let _ = print_entries(entries)?;

    Ok(String::from("Happy Reading!"))
}

fn print_entries<'a, F: FnMut(&rusqlite::Row) -> Entry>(rows: rusqlite::MappedRows<'a, F>)
                                                        -> Result<u32> {
    let mut table = Table::new();
    table.add_row(row!["ID", "Name", "AUTHOR", "GENRE", "PAGE", "READ", "DATE ADDED",
    "DATE FINISHED", "RATING"]);

    let mut n = 0;

    let get_time = |ts| {
        strftime(DATE_FMT, &at(ts)).expect("Bad date formatting")
    };

    for entry in rows {
        let entry = entry?;

        n += 1;

        let mut row = row![&entry.id.to_string(),
                       &entry.name.to_string(),
                       &entry.author.to_string(),
                       &entry.genre.to_string(),
                       &entry.page.to_string(),
                       &entry.read.to_string(),
                       get_time(entry.date_added)];

        if let Some(read_date) = entry.date_finished {
            row.add_cell(Cell::new(&get_time(read_date)));
        } else {
            row.add_cell(cell!());
        }

        if let Some(ref rating) = entry.rating {
            row.add_cell(Cell::new(&rating.to_string()));
        }

        table.add_row(row);
    }

    table.printstd();

    Ok(n)
}
