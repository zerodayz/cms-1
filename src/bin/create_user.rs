#[path = "../db/mod.rs"]
mod db;
use db::utils::*;
use db::users::*;

#[path = "../config/mod.rs"]
mod config;
use config::config::CONFIG;

#[path = "../auth/mod.rs"]
mod auth;
use auth::utils::*;

use std::io::stdin;

fn main() {
    println!("Enter username:");
    let mut username = String::new();
    stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    println!("Enter password:");
    let mut password = String::new();
    stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    // create hash from password
    let hash = hash(&password);

    println!("Is this user going to be admin (y/n)? ");
    let mut is_admin = String::new();
    stdin().read_line(&mut is_admin).unwrap();
    let is_admin = is_admin.trim_end(); // Remove the trailing newline

    if is_admin == "y" {
        let user = create_user(username, &hash[..], true);
        println!("Created user {} with id {}", username, user.id);
    } else {
        let user = create_user(username, &hash[..], false);
        println!("Created user {} with id {}", username, user.id);
    }
}