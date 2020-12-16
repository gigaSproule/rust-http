#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::sync::Mutex;

use rocket::{response, Response, Rocket, State};
use rocket::http::{RawStr, Status};
use rocket::http::ContentType;
use rocket::http::hyper::Error;
use rocket::response::content::{Plain, Json};
use rusqlite::{Connection, NO_PARAMS, params, Result};

struct Visitor {
    name: String,
    count: i32,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/<name>")]
fn hello(connection: State<Mutex<Connection>>, name: &RawStr) -> std::result::Result<Json<String>, Error> {
    let name_str = name.as_str();
    let lock_result = connection.inner().lock();
    if lock_result.is_err() {
        return internal_server_error(format!("Error occurred {}", lock_result.unwrap_err()));
    }
    let conn = lock_result.unwrap();
    let statement_result = conn.prepare("SELECT name, count FROM visitors WHERE name = ?1");
    if statement_result.is_err() {
        return internal_server_error(format!("Error occurred {}", statement_result.unwrap_err()));
    }
    let mut statement = statement_result.unwrap();
    let rows = statement.query_map(params![name_str], |row| {
        Ok(Visitor {
            name: row.get(0).unwrap(),
            count: row.get(1).unwrap(),
        })
    });
    if rows.is_err() {
        return internal_server_error(format!("Error occurred {}", rows.err().unwrap()));
    }
    let previous = rows.unwrap().next();
    let mut count = 1;
    if previous.is_some() {
        let previous_result = previous.as_ref().unwrap();
        if previous_result.is_err() {
            return internal_server_error(format!("Error occurred {}", previous_result.as_ref().err().unwrap()));
        }
        count = previous_result.as_ref().unwrap().count + 1;
        let update_result = conn.execute("UPDATE visitors SET count = count + 1 WHERE name = ?1", params![name_str]);
        if update_result.is_err() {
            return internal_server_error(format!("Error occurred {}", update_result.err().unwrap()));
        }
    } else {
        let insert_result = conn.execute("INSERT INTO visitors (name, count) VALUES (?1, ?2)", params![name_str, count]);
        if insert_result.is_err() {
            return internal_server_error(format!("Error occurred {}", insert_result.err().unwrap()));
        }
    }
    ok(format!("Hello, {}. Previous visits were {}, now it's {} times!", name_str, previous.unwrap_or(Ok(Visitor { name: "".to_string(), count: 0 })).unwrap().count, count))
}

fn internal_server_error<'a>(message: String) -> std::result::Result<Response<'a>, Error> {
    response::Response::build()
        .header(ContentType::Plain)
        .sized_body(Json(message))
        .status(Status::InternalServerError)
        .ok()
}

fn ok<'a>(message: String) -> std::result::Result<Response<'a>, Error> {
    response::Response::build()
        .header(ContentType::Plain)
        .sized_body(Plain(message))
        .status(Status::Ok)
        .ok()
}

fn main() -> Result<()> {
    let connection = Connection::open_in_memory()?;
    connection.execute("CREATE TABLE IF NOT EXISTS visitors (id INTEGER AUTO INCREMENT PRIMARY KEY, name TEXT NOT NULL, count INTEGER)", NO_PARAMS)?;
    let safe_connection = Mutex::new(connection);
    rocket(safe_connection).launch();
    Ok(())
}

fn rocket(safe_connection: Mutex<Connection>) -> Rocket {
    rocket::ignite().manage(safe_connection).mount("/", routes![index, hello])
}

#[cfg(test)]
mod tests {
    use Connection;
    use rocket::http::Status;
    use rocket::local::Client;

    use super::*;

    #[test]
    fn index_returns_hello_world() {
        let client = Client::new(rocket(Mutex::new(Connection::open_in_memory().unwrap()))).expect("valid rocket instance");
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
    }

    #[test]
    fn hello_returns_error_when_visitors_table_not_there() {
        let connection = Connection::open_in_memory().unwrap();
        let client = Client::new(rocket(Mutex::new(connection))).expect("valid rocket instance");
        let mut response = client.get("/name").dispatch();
        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.body_string(), Some("Hello, name. Previous visits were 0, now it's 1 times!".into()));
    }

    #[test]
    fn hello_returns_param_with_counter() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute("CREATE TABLE IF NOT EXISTS visitors (id INTEGER AUTO INCREMENT PRIMARY KEY, name TEXT NOT NULL, count INTEGER)", NO_PARAMS);
        let client = Client::new(rocket(Mutex::new(connection))).expect("valid rocket instance");
        let mut response = client.get("/name").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, name. Previous visits were 0, now it's 1 times!".into()));
    }
}