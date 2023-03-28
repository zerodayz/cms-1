use diesel::{QueryDsl, RunQueryDsl};
use crate::db::models::{NewUser, User};
use diesel::prelude::*;
use crate::db::utils::*;
use crate::config::config::CONFIG;
use crate::db::schema::users;
use crate::db::schema::users::dsl::*;

pub fn create_user(_username: &str, _password: &str, _is_admin: bool) -> User {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let new_user = NewUser {
                username: _username,
                password: _password,
                is_admin: _is_admin
            };

            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(conn)
                .expect("Error saving new user");

            let result = users
                .filter(username.eq(_username))
                .first::<User>(conn)
                .expect("Error loading user");

            let user = result.clone();
            user
        },
        Err(_) => {
            println!("Database connection error.");
            User {
                id: 0,
                username: "".to_string(),
                password: "".to_string(),
                is_admin: false
            }
        },
    }
}

pub fn get_user<'a>(by: &str, value: &str) -> Result<Option<User>, &'a str> {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = match by {
                "id" => users.filter(id.eq(value.parse::<i32>().unwrap_or(0))).load::<User>(conn),
                "username" => users.filter(username.eq(value)).load::<User>(conn),
                _ => users.filter(id.eq(0)).load::<User>(conn)
            }.expect("Error loading user");
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

pub fn delete_user(_user_id: i32) {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            diesel::delete(users.find(_user_id))
                .execute(conn)
                .expect("Error deleting user");
        },
        Err(_) => {
            println!("Database connection error.");
        },
    }
}