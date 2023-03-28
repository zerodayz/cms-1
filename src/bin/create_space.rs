#[path = "../db/mod.rs"]
mod db;
use db::utils::*;
use db::spaces::*;

#[path = "../config/mod.rs"]
mod config;
use config::config::CONFIG;

#[path = "../auth/mod.rs"]
mod auth;
use auth::utils::*;

use std::io::stdin;

fn main() {
    println!("Enter name:");
    let mut name = String::new();
    stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    println!("Enter long name:");
    let mut long_name = String::new();
    stdin().read_line(&mut long_name).unwrap();
    let long_name = long_name.trim();

    println!("Enter owner user ID:");
    let mut owner_id = String::new();
    stdin().read_line(&mut owner_id).unwrap();
    let owner_id = owner_id.trim().parse::<i32>().unwrap();

    println!("Is this space going to be public (y/n)? ");
    let mut is_public = String::new();
    stdin().read_line(&mut is_public).unwrap();
    let is_public = is_public.trim_end(); // Remove the trailing newline

    if is_public == "y" {
        let space = create_space(name, long_name, owner_id, true);
        println!("Created space {}", name);
    } else {
        let space = create_space(name, long_name, owner_id, false);
        println!("Created space {}", name);
    }
}