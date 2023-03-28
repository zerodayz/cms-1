use diesel::{RunQueryDsl};
use crate::db::models::{NewSession, Session};
use diesel::prelude::*;
use crate::db::utils::*;
use crate::config::config::CONFIG;
use crate::db::schema::sessions;
use crate::db::schema::sessions::dsl::*;
use crate::auth::utils::*;

pub fn get_or_create_session(_user_id: i32) -> String {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            // delete any existing sessions for this user
            diesel::delete(sessions.filter(user_id.eq(_user_id)))
                .execute(conn)
                .expect("Error deleting session");

            // create new one
            let one_day = 3600 * 24;
            let _start_date = chrono::NaiveDateTime::from_timestamp_opt(chrono::Local::now().timestamp(), 0);
            let _end_date = chrono::NaiveDateTime::from_timestamp_opt(chrono::Local::now().timestamp() + one_day, 0);

            // generate a random session id
            let binding = generate_token();
            let _session_id = binding.trim_end();

            let new_session = NewSession {
                user_id: _user_id,
                session_id: _session_id,
                start_date: _start_date,
                end_date: _end_date,
            };

            diesel::insert_into(sessions::table)
                .values(&new_session)
                .execute(conn)
                .expect("Error saving new session");

            binding
        },
        Err(_) => {
            println!("Database connection error.");
            "".to_string()
        },
    }
}

pub fn get_session_by_user(_user_id: i32) -> String {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            let result = sessions.filter(user_id.eq(_user_id))
                .first::<Session>(conn)
                .optional()
                .expect("Error loading session");

            match result {
                Some(session) => {
                    session.session_id
                },
                None => {
                    "".to_string()
                },
            }
        },
        Err(_) => {
            println!("Database connection error.");
            "".to_string()
        },
    }
}

pub fn delete_session_by_user(_user_id: i32) {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            diesel::delete(sessions.filter(user_id.eq(_user_id)))
                .execute(conn)
                .expect("Error deleting session");
        },
        Err(_) => {
            println!("Database connection error.");
        },
    }
}

pub fn delete_session(_session_id: i32) {
    let conn = &mut establish_connection(&*CONFIG);
    match conn {
        Ok(conn) => {
            diesel::delete(sessions.find(_session_id))
                .execute(conn)
                .expect("Error deleting session");
        },
        Err(_) => {
            println!("Database connection error.");
        },
    }
}