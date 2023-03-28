use diesel::{QueryDsl, RunQueryDsl};
use crate::db::models::{NewSpace, Post, Space};
use diesel::prelude::*;
use crate::db::utils::*;
use crate::config::config::CONFIG;
use crate::db::schema::spaces;
use crate::db::schema::spaces::dsl::*;
use crate::db::schema::spaces::dsl::id;
use crate::db::schema::posts::dsl::*;

pub fn create_space(_name: &str, _long_name: &str, _owner_id: &str, _is_public: bool) -> Space {
    let _owner_id = _owner_id.parse::<i32>().unwrap();
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let new_space = NewSpace {
                name: _name,
                long_name: _long_name,
                owner_id: _owner_id,
                is_public: _is_public
            };

            diesel::insert_into(spaces::table)
                .values(&new_space)
                .execute(conn)
                .expect("Error saving new space");

            let result = spaces
                .filter(name.eq(_name))
                .load::<Space>(conn)
                .expect("Error loading spaces");

            let space = result.get(0).unwrap().clone();
            space
        },
        Err(_) => {
            println!("Database connection error.");
            Space {
                id: 0,
                name: "".to_string(),
                long_name: "".to_string(),
                owner_id: 0,
                is_public: false
            }
        },
    }
}

pub fn get_space_by_id(_space_id: i32) -> Space {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = spaces
                .filter(id.eq(_space_id))
                .load::<Space>(conn)
                .expect("Error loading spaces");

            let space = result.get(0).unwrap_or(&Space {
                id: 0,
                name: "".to_string(),
                long_name: "".to_string(),
                owner_id: 0,
                is_public: false
            }).clone();

            space
        },
        Err(_) => {
            println!("Database connection error.");
            Space {
                id: 0,
                name: "".to_string(),
                long_name: "".to_string(),
                owner_id: 0,
                is_public: false
            }
        },
    }
}

pub fn get_spaces_by_user_id(_owner_id: &str) -> Vec<Space> {
    let _owner_id = _owner_id.parse::<i32>().unwrap();
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = spaces
                .filter(owner_id.eq(_owner_id))
                .load::<Space>(conn)
                .expect("Error loading spaces");
            result
        },
        Err(_) => {
            println!("Database connection error.");
            Vec::new()
        },
    }
}

pub fn get_space_by_name(_name: &str) -> Space {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = spaces
                .filter(name.eq(_name))
                .load::<Space>(conn)
                .expect("Error loading spaces");

            let space = result.get(0).unwrap_or(&Space {
                id: 0,
                name: "".to_string(),
                long_name: "".to_string(),
                owner_id: 0,
                is_public: false
            }).clone();

            space
        },
        Err(_) => {
            println!("Database connection error.");
            Space {
                id: 0,
                name: "".to_string(),
                long_name: "".to_string(),
                owner_id: 0,
                is_public: false
            }
        },
    }
}

pub fn get_posts_by_space_id(_space_id: i32) -> Vec<Post> {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = posts
                .filter(space_id.eq(_space_id))
                .order(updated_at.desc())
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

pub fn delete_space(_space_id: i32) {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            diesel::delete(spaces.find(_space_id))
                .execute(conn)
                .expect("Error deleting space");
        },
        Err(_) => {
            println!("Database connection error.");
        },
    }
}