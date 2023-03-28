use diesel::prelude::*;

use crate::db::schema::posts;
use crate::db::schema::users;
use crate::db::schema::spaces;
use crate::db::schema::sessions;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Page {
    pub title: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Error {
    pub message: Option<String>,
}

#[derive(Queryable, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub is_admin: bool,
}

#[derive(Queryable, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct Space {
    pub id: i32,
    pub name: String,
    pub long_name: String,
    pub owner_id: i32,
    pub is_public: bool,
}

#[derive(Insertable)]
#[diesel(table_name = spaces)]
pub struct NewSpace<'a> {
    pub name: &'a str,
    pub long_name: &'a str,
    pub owner_id: i32,
    pub is_public: bool,
}

#[derive(Queryable, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub space_id: i32,
    pub title: String,
    pub uri: String,
    pub body: String,
    pub published: bool,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub user_id: i32,
    pub space_id: i32,
    pub title: &'a str,
    pub uri: &'a str,
    pub body: &'a str,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub session_id: String,
    pub start_date: Option<chrono::NaiveDateTime>,
    pub end_date: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    pub user_id: i32,
    pub session_id: &'a str,
    pub start_date: Option<chrono::NaiveDateTime>,
    pub end_date: Option<chrono::NaiveDateTime>,
}