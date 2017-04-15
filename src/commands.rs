use prettytable::Table;

use rusqlite::{self, Connection};

use data::Entry;
use err::TErr;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add(Entry),
    Search(String),
    List,
    Finish(u32),
    Delete(u32),
    SetPage(u32, u32),
    Choose,
    Nil,
}

pub fn exec_command(conn: &Connection, command: Command) -> Result<String, TErr> {
    match command {
        Command::Add(ref entry) => do_add(conn, entry),
        Command::List => do_list(conn),
        Command::Search(ref term) => do_search(conn, term),
        Command::Finish(id) => do_finish(conn, id),
        Command::Delete(id) => do_delete(conn, id),
        Command::SetPage(id, page) => do_set_page(conn, id, page),
        Command::Choose => do_choose(conn),
        Command::Nil => Ok(String::new()),
    }
}

fn do_add(conn: &Connection, entry: &Entry) -> Result<String, TErr> {
    let _ = conn.execute("INSERT INTO entry (name, author, read, page, genre)
                  VALUES (?1, ?2, ?3, ?4, ?5)",
                         &[&entry.name,
                           &entry.author,
                           &entry.read,
                           &entry.page,
                           &entry.genre])?;

    Ok(format!("Successfully added {} by {}", entry.name, entry.author))
}

fn do_list(conn: &Connection) -> Result<String, TErr> {
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

    let _ = print_entries(entries)?;

    Ok("End of List".into())
}

fn do_search(conn: &Connection, term: &str) -> Result<String, TErr> {
    let mut stmt = conn.prepare("SELECT id, name, author, page, genre, read FROM entry
                                 WHERE name LIKE ?1 OR author LIKE ?1 or genre LIKE ?1
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
            }
        })?;

    let n = print_entries(entries)?;

    Ok(format!("Found {} results", n))
}

fn do_finish(conn: &Connection, id: u32) -> Result<String, TErr> {
    let mut stmt = conn.prepare("UPDATE entry SET read = 1 WHERE id = ?1")?;

    let num_updated = stmt.execute(&[&id])?;

    let good = num_updated > 0;

    if good {
        Ok(format!("Entry {} marked as read", id))
    } else {
        Err(TErr::NoSuchEntry(id))
    }
}

fn do_delete(conn: &Connection, id: u32) -> Result<String, TErr> {
    let mut stmt = conn.prepare("DELETE FROM entry WHERE id = ?1")?;

    let num_deleted = stmt.execute(&[&id])?;

    if num_deleted > 0 {
        Ok(format!("Entry {} deleted", id))
    } else {
        Err(TErr::NoSuchEntry(id))
    }
}

fn do_set_page(conn: &Connection, id: u32, page: u32) -> Result<String, TErr> {
    let mut stmt = conn.prepare("UPDATE entry SET page = ?2 where id = ?1")?;

    let num_updated = stmt.execute(&[&id, &page])?;

    if num_updated > 0 {
        Ok(format!("Set page to {} for entry {}", page, id))
    } else {
        Err(TErr::NoSuchEntry(id))
    }
}
fn do_choose(conn: &Connection) -> Result<String, TErr> {
    let mut stmt = conn.prepare("SELECT id, name, author, page, genre, read FROM entry
                                 WHERE read = 0
                                 ORDER BY RANDOM()
                                 LIMIT 1")?;

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

    let _ = print_entries(entries)?;

    Ok(String::from("Happy Reading!"))
}

fn print_entries<'a, F: FnMut(&rusqlite::Row) -> Entry>(rows: rusqlite::MappedRows<'a, F>)
                                                        -> Result<u32, TErr> {
    let mut table = Table::new();
    table.add_row(row!["ID", "Name", "AUTHOR", "GENRE", "PAGE", "READ"]);

    let mut n = 0;

    for entry in rows {
        let entry = entry?;

        n += 1;

        let row = row![&entry.id.to_string(),
                       &entry.name.to_string(),
                       &entry.author.to_string(),
                       &entry.genre.to_string(),
                       &entry.page.to_string(),
                       &entry.read.to_string()];

        table.add_row(row);
    }

    table.printstd();

    Ok(n)
}
