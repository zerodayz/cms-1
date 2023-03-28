#[path = "../db/mod.rs"]
mod db;
use db::utils::*;
use db::posts::*;

#[path = "../auth/mod.rs"]
mod auth;
use auth::utils::*;

#[path = "../config/mod.rs"]
mod config;
use config::config::CONFIG;

use std::env::args;

fn main() {
    let post_id = args()
        .nth(1)
        .expect("delete_post requires a post id")
        .parse::<&str>()
        .expect("Invalid ID");

    delete_post(post_id);
}