# SQL Macros

WORK IN PROGRESS

## Base idea

A Sans-IO style system to allow typed queries remaining agnosting on the unerlying database driver.

Since on database world SQL is the de-facto standard language, it doesn't make sense to reinvent the wheel trying to mimic SQL with Rust (or any other language).

Simply define table structs reading `CREATE TABLE` commands, manage migrations within the same query block, so migrations aren't scattered around but live inside the code beside the same table creation.

Select statement will rely on table definition, so the table type must me used locally.

## Examples

### Table creation and migrations

```rust
table!("
CREATE TABLE users( 
    id int NOT NULL PRIMARY KEY AUTO_INCREMENT,
    username varchar(25) NOT NULL,
    password varchar(30) NOT NULL
);

ALTER TABLE users ADD COLUMN disabled BOOL;
")
```

This will generate something like (TBD)
```rust
pub struct User {
    id: i32,
    username: String,
    password: String,
    disabled: Option<bool>,
}

pub trait UserSchema {
    type Id = i32;
    type Username = String;
    type Password = String;
    type Disabled = bool;
}

impl Schema for User {
    type Schema = UserSchema;
}
```

### Select

```rust
select!(connection, "SELECT id, username FROM users WHERE disabled = true", Row)
```

This will generate something like (TBD)
```rust
{
    let rows = query_executor(connection, "SELECT id, username FROM users WHERE disabled = true").await?;
    Ok(Row {
        id: UserSchema::Id::parse(rows)?,
        username: UserSchema::Username::parse(rows)?,
    })
}
```
