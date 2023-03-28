#[path = "../db/mod.rs"]
mod db;
use db::utils::*;
use db::sessions::*;

#[path = "../auth/mod.rs"]
mod auth;
use auth::utils::*;

#[path = "../config/mod.rs"]
mod config;
use config::config::CONFIG;

use std::env::args;

fn main() {
    let session_id = args()
        .nth(1)
        .expect("delete_session requires a session id")
        .parse::<i32>()
        .expect("Invalid ID");

    delete_session(session_id);
}