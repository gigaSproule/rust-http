#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::http::RawStr;
use rocket::State;

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

#[get("/<name>")]
fn hello(counter: State<Arc<Mutex<Counter>>>, name: &RawStr) -> String {
    let count = counter.inner().lock().unwrap().increment(name.as_str());
    format!("Hello, {} (for the {} time)!", name.as_str(), count)
}

fn main() {
    let mut counter = Arc::new(Mutex::new(Counter::new()));
    rocket::ignite().manage(counter).mount("/", routes![index, hello]).launch();
}
