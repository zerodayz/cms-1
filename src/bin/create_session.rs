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

use std::io::stdin;

fn main() {
    println!("Enter user id:");
    let mut user_id = String::new();
    stdin().read_line(&mut user_id).unwrap();
    let user_id = user_id.trim().parse::<i32>().unwrap();

    let session = get_or_create_session(user_id);
    println!("Created session {} for user_id {}", session, user_id);
}