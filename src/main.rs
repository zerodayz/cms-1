#[macro_use] extern crate rocket;

use rocket::fs::FileServer;

extern crate rocket_dyn_templates;
use rocket_dyn_templates::{Template, Engines};
use rocket::Request;
use rocket::response::Redirect;

use crate::responses::responses::{AnyResponse, Context};

pub mod db;
pub mod config;
pub mod auth;
pub mod pages;
pub mod utils;
pub mod filters;
pub mod responses;

use crate::db::models::*;
use crate::db::users;
use crate::pages::login as login;
use crate::pages::space as space;
use crate::pages::posts as posts;
use crate::auth::utils as auth_utils;


// Catches Bad Request
//
#[catch(400)]
fn bad_request(request: &Request) -> AnyResponse {
    let mut context = Context::new();
    let cookies = request.cookies();
    if auth_utils::verify_session(cookies) {
        let user_id = cookies.get("cms_user_id").unwrap().value();
        let user_data = users::get_user("id", user_id);
        match user_data {
            Ok(Some(user_data)) => {
                context.set_user(user_data);
            },
            Ok(None) => {}
            Err(_) => {}
        }
    }
    let page = Page {
        title: "400 Bad Request".to_string(),
    };
    context.set_page(page);
    let error = Error {
        message: Option::from("The page you are looking for does not exist.".to_string()),
    };
    context.set_error(error);
    return AnyResponse::Template(Template::render("404", &context))
}

// Catches all 404s and redirects to the 404 page
//
#[catch(404)]
fn not_found(request: &Request) -> AnyResponse {
    let cookies = request.cookies();
    let mut context = Context::new();
    if auth_utils::verify_session(cookies) {
        let user_id = cookies.get("cms_user_id").unwrap().value();
        let user_data = users::get_user("id", user_id);
        match user_data {
            Ok(Some(user_data)) => {
                context.set_user(user_data);
            },
            Ok(None) => {}
            Err(_) => {}
        }
    }
    let page = Page {
        title: "404 Not Found".to_string(),
    };
    context.set_page(page);
    let error = Error {
        message: Option::from("The page you are looking for does not exist.".to_string()),
    };
    context.set_error(error);
    return AnyResponse::Template(Template::render("404", &context))
}

// Catches Unprocessable Entity
//
#[catch(422)]
fn unprocessable_entity(request: &Request) -> AnyResponse {
    let cookies = request.cookies();
    let mut context = Context::new();
    if auth_utils::verify_session(cookies) {
        let user_id = cookies.get("cms_user_id").unwrap().value();
        let user_data = users::get_user("id", user_id);
        match user_data {
            Ok(Some(user_data)) => {
                context.set_user(user_data);
            },
            Ok(None) => {}
            Err(_) => {}
        }
    }
    let page = Page {
        title: "422 Unprocessable Entity".to_string(),
    };
    context.set_page(page);
    let error = Error {
        message: Option::from("The page you are looking for does not exist.".to_string()),
    };
    context.set_error(error);
    return AnyResponse::Template(Template::render("404", &context))
}

// Catches all 500s and redirects to the 404 page
//
#[catch(500)]
fn internal_server_error(request: &Request) -> AnyResponse {
    let cookies = request.cookies();
    let mut context = Context::new();
    if auth_utils::verify_session(cookies) {
        let user_id = cookies.get("cms_user_id").unwrap().value();
        let user_data = users::get_user("id", user_id);
        match user_data {
            Ok(Some(user_data)) => {
                context.set_user(user_data);
            },
            Ok(None) => {}
            Err(_) => {}
        }
    }
    let page = Page {
        title: "500 Internal Server Error".to_string(),
    };
    context.set_page(page);
    let error = Error {
        message: Option::from("The page you are looking for does not exist.".to_string()),
    };
    context.set_error(error);
    return AnyResponse::Template(Template::render("404", &context))
}


#[get("/")]
fn index() -> AnyResponse { AnyResponse::Redirect(Redirect::to("/home")) }

#[get("/home")]
fn home() -> AnyResponse { AnyResponse::Redirect(Redirect::to("/spaces/home")) }


// https://rocket.rs/v0.5-rc/guide/overview/#launching
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index,
            home,
            login::login, login::login_post, login::logout,
            space::space, space::new, space::new_space, space::list,
            posts::new, posts::new_post, posts::list, posts::get_post, posts::delete_post,
            posts::get_post_raw])
        .mount("/public", FileServer::from("static"))
        .register("/", catchers![not_found, internal_server_error, unprocessable_entity, bad_request])
        .attach(Template::custom(|engines: &mut Engines| {
            // add multiple filters
            engines.tera.register_filter("date_ago", filters::filter::date_ago);
            engines.tera.register_filter("user_username", filters::filter::user_username)
        }))
        .launch()
        .await?;
    Ok(())
}
