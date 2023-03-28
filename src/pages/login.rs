use rocket_dyn_templates::{context, Template};
use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar};

use crate::db::sessions::*;
use crate::db::models::*;

use rocket::form::Form;

use crate::auth::utils as auth_utils;
use crate::responses::responses::{AnyResponse, Context};

#[derive(FromForm)]
pub struct LoginForm<'f> {
    username: &'f str,
    password: &'f str,
    remember_me: bool
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> AnyResponse {
    if auth_utils::verify_session(cookies) {
        let user_id = cookies.get("cms_user_id");
        let user_id = user_id.unwrap().value().parse::<i32>().unwrap();
        delete_session_by_user(user_id);
        cookies.remove(Cookie::named("cms_session"));
        cookies.remove(Cookie::named("cms_user_id"));
    }
    AnyResponse::Redirect(Redirect::to("/home"))
}

#[get("/login")]
pub fn login(cookies: &CookieJar<'_>) -> AnyResponse {
    if auth_utils::verify_session(cookies) {
        // If the user is already logged in, redirect them to the home page
        //
        return AnyResponse::Redirect(Redirect::to("/home"));
    }
    return AnyResponse::Template(Template::render("login", &context!{}))
}

// Sign In Page
#[post("/login", data = "<login_form>")]
pub fn login_post(cookies: &CookieJar<'_>, login_form: Form<LoginForm>) -> AnyResponse {
    // Verify the credentials are valid
    //
    let user = auth_utils::verify_login(login_form.username, login_form.password);
    return match user {
        Ok(Some(user)) => {
            let session = get_or_create_session(user.id);
            // If the user checked the "remember me" box, set the session cookie to expire in 30 days
            //
            auth_utils::build_session_cookie(cookies, session, &user, login_form.remember_me);
            AnyResponse::Redirect(Redirect::to("/home"))
        },
        Err(err) => {
            // Login failed, render the login page with relevant message
            //
            let mut context = Context::new();
            let error = Error {
                message: Option::from(err.to_string()),
            };
            context.set_error(error);
            AnyResponse::Template(Template::render("login", &context))
        }
        _ => { AnyResponse::Template(Template::render("login", &context!{})) }
    };
}