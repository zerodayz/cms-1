extern crate diesel;
use self::diesel::prelude::*;

#[path = "../db/mod.rs"]
mod db;
use db::models::*;
use db::utils::*;

#[path = "../auth/mod.rs"]
mod auth;
use auth::utils::*;

#[path = "../config/mod.rs"]
mod config;
use config::config::CONFIG;

fn main() {
    use db::schema::users::dsl::*;
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let results = users
                .load::<User>(conn)
                .expect("Error loading users");

            println!("Displaying {} users", results.len());
            for u in results {
                println!("ID : {:?}", u.id);
                println!("Username : {:?}", u.username);
                println!("Password : {:?}", u.password);
                println!("Is Admin : {:?}", u.is_admin);
                println!("----------------------------------------\n");
            }
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}