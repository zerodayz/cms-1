use rocket_dyn_templates::Template;
use rocket::response::Redirect;
use rocket::http::{CookieJar};

use crate::auth::utils as auth_utils;
use crate::responses::responses::{AnyResponse, Context};

use rocket::form::Form;
use slug::slugify;

use crate::db::posts as posts;
use crate::db::spaces as spaces;
use crate::db::models::*;
use crate::db::users;

#[derive(FromForm)]
pub struct PostForm<'f> {
    content: &'f str,
    space: &'f str,
}

#[get("/post/new")]
pub fn new(cookies: &CookieJar<'_>) -> AnyResponse {
    if !auth_utils::verify_session(cookies) {
        return AnyResponse::Redirect(Redirect::to("/login"))
    }
    // Check if user is owner of a space
    //
    // If not, redirect to /space/new
    //
    let user_id = cookies.get("cms_user_id").unwrap().value();
    let spaces = spaces::get_spaces_by_user_id(user_id);
    if spaces.len() == 0 {
        return AnyResponse::Redirect(Redirect::to("/space/new"))
    }
    let mut context = Context::new();
    let user_data = users::get_user("id", user_id);
    match user_data {
        Ok(Some(user_data)) => {
            let user = User {
                id: user_data.id,
                username: user_data.username,
                password: "".to_string(),
                is_admin: user_data.is_admin,
            };
            context.set_user(user);
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
        title: "Create a new post".to_string(),
    };
    context.set_page(page);
    let spaces = spaces::get_spaces_by_user_id(user_id);
    context.set_spaces(spaces);
    return AnyResponse::Template(Template::render("post/post_new", &context))
}

#[post("/post/new", data = "<post_form>")]
pub fn new_post(cookies: &CookieJar<'_>, post_form: Form<PostForm>) -> AnyResponse {
    if !auth_utils::verify_session(cookies) {
        return AnyResponse::Redirect(Redirect::to("/login"))
    }
    let title = post_form.content.split("<h1>").nth(1).unwrap().split("</h1>").nth(0).unwrap();
    let content = post_form.content.split("<h1>").nth(1).unwrap().split("</h1>").nth(1).unwrap();
    // slugify title into uri
    //
    let uri = slugify(title);
    let space = spaces::get_space_by_name(post_form.space);
    let user_id = cookies.get("cms_user_id").unwrap().value();
    posts::create_post(title, &uri, &content, user_id, space.id);
    return AnyResponse::Redirect(Redirect::to("/spaces/".to_string() + &space.name))
}

#[get("/post-delete/<id>")]
pub fn delete_post(cookies: &CookieJar<'_>, id: String) -> AnyResponse {
    if !auth_utils::verify_session(cookies) {
        return AnyResponse::Redirect(Redirect::to("/login"))
    }
    posts::delete_post(&id);
    return AnyResponse::Redirect(Redirect::to("/post/list"))
}

#[get("/post/list")]
pub fn list(cookies: &CookieJar<'_>) -> AnyResponse {
    if !auth_utils::verify_session(cookies) {
        return AnyResponse::Redirect(Redirect::to("/login"))
    }

    let mut context = Context::new();
    let page = Page {
        title: "My Posts".to_string(),
    };
    context.set_page(page);

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
    let posts = posts::get_posts_by_user_id(user_id);
    context.set_posts(posts);
    return AnyResponse::Template(Template::render("post/post_list", &context))
}

// Get post raw content
//
#[get("/post/<uri>/raw")]
pub fn get_post_raw(cookies: &CookieJar<'_>, uri: String) -> AnyResponse {
    let post = posts::get_post_by_uri(&uri);
    match post {
        Ok(Some(post)) => {
            // Check if the post is in public space
            //
            let space = spaces::get_space_by_id(post.space_id);
            if space.is_public != true {
                if !auth_utils::verify_session(cookies) {
                    return AnyResponse::Redirect(Redirect::to("/login"))
                }
            }
            let mut context = Context::new();
            let page = Page {
                title: post.title.to_string(),
            };
            context.set_page(page);
            context.set_post(post);
            if auth_utils::verify_session(cookies) {
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
            }
            return AnyResponse::Template(Template::render("post/post_raw", &context))
        },
        Ok(None) => {
            return AnyResponse::Redirect(Redirect::to("/"))
        }
        Err(err) => {
            let error = Error {
                message: Option::from(err.to_string()),
            };
            let mut context = Context::new();
            context.set_error(error);
            return AnyResponse::Template(Template::render("error", &context))
        }
    }
}

// Get post by uri
//
#[get("/post/<uri>")]
pub fn get_post(cookies: &CookieJar<'_>, uri: String) -> AnyResponse {
    let post = posts::get_post_by_uri(&uri);
    match post {
        Ok(Some(post)) => {
            // Check if the post is in public space
            //
            let space = spaces::get_space_by_id(post.space_id);
            if space.is_public != true {
                if !auth_utils::verify_session(cookies) {
                    return AnyResponse::Redirect(Redirect::to("/login"))
                }
            }
            let mut context = Context::new();
            let page = Page {
                title: post.title.to_string(),
            };
            context.set_page(page);
            context.set_post(post);
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
            return AnyResponse::Template(Template::render("post/post", &context))
        },
        Ok(None) => {
            return AnyResponse::Redirect(Redirect::to("/post/new"))
        }
        Err(err) => {
            let error = Error {
                message: Option::from(err.to_string()),
            };
            let mut context = Context::new();
            context.set_error(error);
            return AnyResponse::Template(Template::render("error", &context))
        }
    }
}