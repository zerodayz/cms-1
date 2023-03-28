#[path = "../db/mod.rs"]
mod db;
use db::utils::*;
use db::users::*;

#[path = "../auth/mod.rs"]
mod auth;
use auth::utils::*;

#[path = "../config/mod.rs"]
mod config;
use config::config::CONFIG;

use std::env::args;

fn main() {
    let user_id = args()
        .nth(1)
        .expect("delete_user requires a user id")
        .parse::<i32>()
        .expect("Invalid ID");

    delete_user(user_id);
    println!("Deleted user with user_id {}", user_id);
}