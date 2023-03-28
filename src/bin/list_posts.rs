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
    use db::schema::posts::dsl::*;
    let config = &*CONFIG;
    let connection = &mut establish_connection(config);
    match connection {
        Ok(conn) => {
            let results = posts
                .load::<Post>(conn)
                .expect("Error loading posts");

            println!("Displaying {} posts", results.len());
            for p in results {
                println!("ID : {:?}", p.id);
                println!("Title : {:?}", p.title);
                println!("Title Description: {:?}", p.title_description);
                println!("Body : {:?}", p.body);
                println!("User ID : {:?}", p.user_id);
                println!("Published : {:?}", p.published);
                println!("Updated At : {:?}", p.updated_at);
                println!("----------------------------------------\n");
            }
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}