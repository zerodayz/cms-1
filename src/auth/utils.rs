use crate::db::{sessions, users};

use thiserror::Error;
use rand::Rng;
use rand::distributions::Alphanumeric;
use bcrypt;

// rocket cookies
use rocket::http::{Cookie, CookieJar};
use rocket::time::Duration;
use crate::db::models::User;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown error.")]
    Unknown,
    #[error("Invalid username.")]
    InvalidUsername,
    #[error("Invalid password.")]
    InvalidPassword,
    #[error("Database connection error.")]
    DatabaseConnectionError,
}

pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let token: String = (0..255).map(|_| rng.sample(Alphanumeric) as char).collect();
    token
}

pub fn verify_session(cookies: &CookieJar<'_>) -> bool {
    // Get cookie
    let session = cookies.get("cms_session");
    let user_id = cookies.get("cms_user_id");

    // Verify if session is still valid
    if session.is_some() {
        let session = session.unwrap().value();
        let user_id = user_id.unwrap().value().parse::<i32>().unwrap();
        let existing_session = sessions::get_session_by_user(user_id);
        if existing_session != "" && (session == existing_session) {
            return true
        }
    }
    return false
}

pub fn build_session_cookie(cookies: &CookieJar<'_>, session: String, user: &User, remember_me: bool) {
    let duration = if remember_me { Duration::days(30) } else { Duration::days(1) };
    let session_cookie = Cookie::build("cms_session", session)
        .path("/")
        .max_age(duration)
        .finish();
    let userid_cookie = Cookie::build("cms_user_id", user.id.to_string())
        .path("/")
        .max_age(duration)
        .finish();
    cookies.add(userid_cookie);
    cookies.add(session_cookie);
}

pub fn hash(password: &str) -> String {
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    hash
}

pub fn compare_passwords(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap()
}

pub fn verify_login(username: &str, password: &str) -> Result<Option<User>, Error> {
    let user = users::get_user("username", username);
    return match user {
        Ok(Some(user)) => {
            if compare_passwords(password, &user.password) {
                Ok(Some(user))
            } else {
                Err(Error::InvalidPassword)
            }
        },
        Ok(None) => Err(Error::InvalidUsername),
        Err(_) => Err(Error::DatabaseConnectionError),
    }
}