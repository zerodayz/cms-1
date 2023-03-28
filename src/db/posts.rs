use diesel::{RunQueryDsl};
use crate::db::models::{NewPost, Post};
use diesel::prelude::*;
use crate::db::utils::*;
use crate::config::config::CONFIG;
use crate::db::schema::posts;
use crate::db::schema::posts::dsl::*;

pub fn create_post(_title: &str, _uri: &str, _body: &str, _user_id: &str, _space_id: i32) {
    let _user_id = _user_id.parse::<i32>().unwrap();
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let _updated_at = chrono::NaiveDateTime::from_timestamp_opt(chrono::Local::now().timestamp(), 0);

            let new_post = NewPost {
                title: _title,
                uri: _uri,
                body: _body,
                user_id: _user_id,
                space_id: _space_id,
                updated_at: _updated_at,
            };

            diesel::insert_into(posts::table)
                .values(&new_post)
                .execute(conn)
                .expect("Error saving new post");
        },
        Err(_) => {
            println!("Database connection error.");
        },
    }
}

pub fn get_post_by_uri<'a>(_slug: &str) -> Result<Option<Post>, &'a str> {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = posts
                .filter(uri.eq(_slug))
                .load::<Post>(conn)
                .expect("Error loading posts");
            if result.len() > 0 {
                Ok(Some(result.get(0).unwrap().clone()))
            } else {
                Ok(None)
            }
        },
        Err(_) => {
            Err("Database connection error.")
        },
    }
}

pub fn get_posts_by_user_id(_user_id: &str) -> Vec<Post> {
    let _user_id = _user_id.parse::<i32>().unwrap();
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = posts
                .filter(user_id.eq(_user_id))
                .load::<Post>(conn)
                .expect("Error loading posts");
            result
        },
        Err(_) => {
            println!("Database connection error.");
            Vec::new()
        },
    }
}

pub fn delete_post(_post_id: &str) {
    let _post_id = _post_id.parse::<i32>().unwrap();
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            diesel::delete(posts.find(_post_id))
                .execute(conn)
                .expect("Error deleting post");
        },
        Err(_) => {
            println!("Database connection error.");
        },
    }
}