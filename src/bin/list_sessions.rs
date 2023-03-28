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
    use db::schema::sessions::dsl::*;
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let results = sessions
                .load::<Session>(conn)
                .expect("Error loading sessions");

            println!("Displaying {} sessions", results.len());
            for s in results {
                println!("ID: {:?}", s.id);
                println!("User ID: {:?}", s.user_id);
                println!("Session ID: {:?}", s.session_id);
                println!("Start Date: {:?}", s.start_date);
                println!("End Date: {:?}", s.end_date);
                println!("----------------------------------------\n");
            }
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}