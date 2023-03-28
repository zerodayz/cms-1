extern crate diesel;
use std::env::args;
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
    let post_id = args()
        .nth(1)
        .expect("publish_post requires a post id")
        .parse::<i32>()
        .expect("Invalid ID");
    let connection = &mut establish_connection(config);
    match connection {
        Ok(conn) => {
            let _ = diesel::update(posts.find(post_id))
                .set(published.eq(true));

            let result = posts
                .filter(id.eq(post_id))
                .load::<Post>(conn)
                .expect("Error loading posts");

            let post = result.get(0).unwrap();
            println!("Published post {}", post.title);
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }


}