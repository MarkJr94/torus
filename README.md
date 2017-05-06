# Description

`torus` is simple CLI reading list tracker. It's based on and shameless stolen from a script a friend wrote and used themself, and I mostly wrote this for fun and practice.

# Usage

```
CLI Reading List application

USAGE:
    torus [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add         add entry
    choose      Choose a random entry for you to read
    delete      Delete an entry
    finish      Mark an entry as read
    help        Prints this message or the help of the given subcommand(s)
    list        list entries in order of page
    rate        Rate an entry
    search      find entries. case insensitive match on 'TITLE', 'AUTHOR', and 'GENRE'
    set-page    Set the last page you read for an entry
    shell       Enter interactive mode
```

# Example Session

## You Enter
`torus help add`

## Output

```
torus-add 
add entry

USAGE:
    torus add <TITLE> <AUTHOR> <GENRE> [PAGE]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <TITLE>     title of entry
    <AUTHOR>    Author of work
    <GENRE>     Genre of work
    <PAGE>      Page you are currently at [default: 0]
```

## You enter
`torus add 'The Book' 'Alan W. Watts' Philosophy`
## Output
```
Successfully added The Book by Alan W. Watts
```

## You enter
`torus list`

## Output
```
+----+----------+------------+------------+------+------+-------------------+-------------------+--------+
| ID | Name     | AUTHOR     | GENRE      | PAGE | READ | DATE ADDED        | DATE FINISHED     | RATING |
+----+----------+------------+------------+------+------+-------------------+-------------------+--------+
| 1  | The Book | Alan Watts | Philosophy | 0    | true | 05/06/17 13:13:29 |                   |        |
+----+----------+------------+------------+------+------+-------------------+-------------------+--------+
End of List
```
