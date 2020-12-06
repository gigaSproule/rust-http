#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::sync::Mutex;

use rocket::http::RawStr;
use rocket::State;
use rusqlite::{Connection, NO_PARAMS, params, Result};

struct Counter {
    name_count: HashMap<String, i32>
}

impl Counter {
    fn new() -> Counter {
        Counter { name_count: HashMap::new() }
    }

    fn increment(&mut self, name: &str) -> i32 {
        let i = self.name_count.get(name).get_or_insert(&0).clone() + 1;
        self.name_count.insert(name.to_string(), i);
        i
    }

    fn get(&self, name: &str) -> &i32 {
        self.name_count.get(name).unwrap_or(&0)
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

struct Visitor {
    name: String,
    count: i32,
}

#[get("/<name>")]
fn hello(counter: State<Mutex<Counter>>, connection: State<Mutex<Connection>>, name: &RawStr) -> String {
    let name_str = name.as_str();
    let lock_result = connection.inner().lock();
    if lock_result.is_err() {
        return format!("Error occurred {}", lock_result.unwrap_err());
    }
    let conn = lock_result.unwrap();
    let statement_result = conn.prepare("SELECT name, count FROM visitors WHERE name = ?1");
    if statement_result.is_err() {
        return format!("Error occurred {}", statement_result.unwrap_err());
    }
    let mut statement = statement_result.unwrap();
    let rows = statement.query_map(params![name_str], |row| {
        Ok(Visitor {
            name: row.get(0).unwrap(),
            count: row.get(1).unwrap(),
        })
    });
    if rows.is_err() {
        return format!("Error occurred {}", rows.err().unwrap());
    }
    let previous = rows.unwrap().next();
    let count = counter.inner().lock().unwrap().increment(name_str);
    if previous.is_none() {
        let insert_result = conn.execute("INSERT INTO visitors (name, count) VALUES (?1, ?2)", params![name_str, count]);
        if insert_result.is_err() {
            return format!("Error occurred {}", insert_result.err().unwrap());
        }
    } else {
        let update_result = conn.execute("UPDATE visitors SET count = ?1 WHERE name = ?2", params![count, name_str]);
        if update_result.is_err() {
            return format!("Error occurred {}", update_result.err().unwrap());
        }
    }
    format!("Hello, {}. Previous visits were {}, now it's {} times!", name_str, previous.unwrap_or(Ok(Visitor { name: "".to_string(), count: 0 })).unwrap().count, count)
}

fn main() -> Result<()> {
    let connection = Connection::open_in_memory()?;
    connection.execute("CREATE TABLE IF NOT EXISTS visitors (id INTEGER AUTO INCREMENT PRIMARY KEY, name TEXT NOT NULL, count INTEGER)", NO_PARAMS)?;
    let safe_connection = Mutex::new(connection);
    let counter = Mutex::new(Counter::new());
    rocket::ignite().manage(counter).manage(safe_connection).mount("/", routes![index, hello]).launch();
    Ok(())
}
