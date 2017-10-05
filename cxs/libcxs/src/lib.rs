extern crate indy;
extern crate serde;
extern crate serde_json;
extern crate rand;

use std::path::Path;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;

pub mod api;
pub mod connection;

pub fn create_path(s:&str) -> &Path {
    Path::new(s)
}

