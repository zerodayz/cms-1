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

use std::io::stdin;
use std::io::Read;


fn main() {
    let mut title = String::new();
    let mut title_description = String::new();
    let mut body = String::new();

    println!("Enter user id:");
    let mut user_id = String::new();
    stdin().read_line(&mut user_id).unwrap();
    let user_id = user_id.trim();

    /// enter space id
    println!("Enter space id:");
    let mut space_id = String::new();
    stdin().read_line(&mut space_id).unwrap();
    let space_id = space_id.trim().parse::<i32>().unwrap();

    println!("What would you like your title to be?");
    stdin().read_line(&mut title).unwrap();
    let title = title.trim_end(); // Remove the trailing newline


    println!("What would you like your title description to be?");
    stdin().read_line(&mut title_description).unwrap();
    let title_description = title_description.trim_end(); // Remove the trailing newline

    println!(
        "\nOk! Let's write {} (Press {} when finished)\n",
        title, EOF
    );
    stdin().read_to_string(&mut body).unwrap();

    create_post(title, title_description, &body, user_id, space_id);
    println!("\nSaved draft {}", title);
}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";