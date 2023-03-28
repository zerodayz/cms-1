use rocket_dyn_templates::Template;
use rocket::response::Redirect;
use rocket::http::{CookieJar};
use rocket::form::Form;
use slug::slugify;

use crate::auth::utils as auth_utils;
use crate::db::spaces as spaces;
use crate::db::models::*;
use crate::db::users;

use crate::responses::responses::{AnyResponse, Context};


#[derive(FromForm)]
pub struct SpaceForm<'f> {
    long_name: &'f str,
    is_public: Option<&'f str>,
}

#[get("/space/list")]
pub fn list(cookies: &CookieJar<'_>) -> AnyResponse {
    if !auth_utils::verify_session(cookies) {
        return AnyResponse::Redirect(Redirect::to("/login"))
    }
    let user_id = cookies.get("cms_user_id").unwrap().value();
    let spaces = spaces::get_spaces_by_user_id(user_id);
    let mut context = Context::new();
    let user_data = users::get_user("id", user_id);
    match user_data {
        Ok(Some(user_data)) => {
            context.set_user(user_data);
        },
        Ok(None) => {
            return AnyResponse::Redirect(Redirect::to("/login"))
        }
        Err(err) => {
            let error = Error {
                message: Option::from(err.to_string()),
            };
            context.set_error(error);
            return AnyResponse::Template(Template::render("login", &context))
        }
    }
    let page = Page {
        title: "My Spaces".to_string(),
    };
    context.set_page(page);
    context.set_spaces(spaces);
    return AnyResponse::Template(Template::render("space/space_list", &context))
}

#[get("/space/new")]
pub fn new(cookies: &CookieJar<'_>) -> AnyResponse {
    if !auth_utils::verify_session(cookies) {
        return AnyResponse::Redirect(Redirect::to("/login"))
    }
    let mut context = Context::new();
    let user_id = cookies.get("cms_user_id").unwrap().value();
    let user_data = users::get_user("id", user_id);
    match user_data {
        Ok(Some(user_data)) => {
            context.set_user(user_data);
        },
        Ok(None) => {
            return AnyResponse::Redirect(Redirect::to("/login"))
        }
        Err(err) => {
            let error = Error {
                message: Option::from(err.to_string()),
            };
            context.set_error(error);
            return AnyResponse::Template(Template::render("login", &context))
        }
    }
    let page = Page {
        title: "Create a new space".to_string(),
    };
    context.set_page(page);
    return AnyResponse::Template(Template::render("space/space_new", &context))
}


#[post("/space/new", data = "<space_form>")]
pub fn new_space(cookies: &CookieJar<'_>, space_form: Form<SpaceForm>) -> AnyResponse {
    if !auth_utils::verify_session(cookies) {
        return AnyResponse::Redirect(Redirect::to("/login"))
    }
    let slug = slugify(space_form.long_name);
    let mut context = Context::new();
    let space = spaces::get_space_by_name(slug.as_str());
    if space.id != 0 {
        let page = Page {
            title: "Space already exists".to_string(),
        };
        context.set_page(page);
        let error = Error {
            message: Option::from("The space you are trying to create already exists.".to_string()),
        };
        context.set_error(error);
        return AnyResponse::Template(Template::render("space/space_new", &context))
    }
    let user_id = cookies.get("cms_user_id").unwrap().value();
    if space_form.is_public.is_some() {
        spaces::create_space(&slug, space_form.long_name, user_id, true);
    } else {
        spaces::create_space(&slug, space_form.long_name, user_id, false);
    }
    return AnyResponse::Redirect(Redirect::to("/spaces/".to_string() + slug.as_str()))
}

#[get("/spaces/<space>")]
pub fn space(cookies: &CookieJar<'_>, space: &str) -> AnyResponse {
    // Check if the space exists and if not return 404
    //
    let mut context = Context::new();
    let space = spaces::get_space_by_name(space);
    let space_id = space.id;
    if space_id == 0 {
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
            message: Option::from("Failed to find the space you were looking for.".to_string()),
        };
        context.set_error(error);
        return AnyResponse::Template(Template::render("404", &context))
    }
    // Considering the space exists, check if it's public or not
    //
    if !space.is_public {
        // If it's not public, check if the user is logged in
        //
        if !auth_utils::verify_session(cookies) {
            return AnyResponse::Redirect(Redirect::to("/login"))
        }
        // If the user is logged in, check if the user is the owner of the space
        //
        let user_id = cookies.get("cms_user_id");
        let user_id = user_id.unwrap().value().parse::<i32>().unwrap();
        if user_id != space.owner_id {
            let page = Page {
                title: "403 Forbidden".to_string(),
            };
            context.set_page(page);
            let error = Error {
                message: Option::from("You are not allowed to access this space.".to_string()),
            };
            context.set_error(error);
            return AnyResponse::Template(Template::render("404", &context))
        }
    }
    // If the space is public or the user is the owner of the space, render the space
    //
    // Check if the user is set in the cookies
    //
    if auth_utils::verify_session(cookies) {
        let user_id = cookies.get("cms_user_id").unwrap().value();
        let user_data = users::get_user("id", user_id);
        match user_data {
            Ok(Some(user_data)) => {
                context.set_user(user_data);
            },
            Ok(None) => {}
            Err(err) => {
                let error = Error {
                    message: Option::from(err.to_string()),
                };
                context.set_error(error);
            }
        }
    }
    let page = Page {
        title: space.long_name.to_string(),
    };
    context.set_page(page);
    context.set_space(space);
    let posts = spaces::get_posts_by_space_id(space_id);
    context.set_posts(posts);
    return AnyResponse::Template(Template::render("space/space", &context))
}