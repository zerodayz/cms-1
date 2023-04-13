mod flash;

use axum::{async_trait, extract::{Multipart, FromRequestParts, Form, Path, Query, State, TypedHeader}, headers::{authorization::Bearer, Authorization}, http::{request::Parts, Request, StatusCode}, response::{IntoResponse, Response, Html}, routing::{get, get_service, post}, Json, RequestPartsExt, Router, debug_handler, Server, middleware, Extension, RequestExt};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors};
use once_cell::sync::Lazy;
use csm_core::{
    sea_orm::{Database, DatabaseConnection},
    Mutation as MutationCore, Query as QueryCore,
};
use entity::users::Model as UserModel;
use entity::*;
use flash::{guard_response, get_flash_cookie, post_response, add_cookies, PostResponse, get_token_cookie, login_response, LoginResponse, logout_response, LogoutResponse, Data, FlashData, TokenData};
use migration::{Migrator, MigratorTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_value, Value};
use std::str::FromStr;
use std::{fmt::Display, env, net::SocketAddr};
use std::collections::HashMap;
use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::Redirect;
use chrono::{Duration, TimeZone, Utc};
use tera::Tera;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::services::ServeDir;
use uuid::Uuid;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

fn format_date(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let days = value.as_str().unwrap_or_default();
    let parsed_date = Utc.datetime_from_str(days, "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let now = Utc::now();
    let date = now.signed_duration_since(parsed_date);
    if date.num_days() > 0 {
        if date.num_days() == 1 {
            Ok(to_value(format!("{} day ago", date.num_days())).unwrap())
        } else {
            Ok(to_value(format!("{} days ago", date.num_days())).unwrap())
        }
    } else if date.num_hours() > 0 {
        if date.num_hours() == 1 {
            Ok(to_value(format!("{} hour ago", date.num_hours())).unwrap())
        } else {
            Ok(to_value(format!("{} hours ago", date.num_hours())).unwrap())
        }
    } else if date.num_minutes() > 0 {
        if date.num_minutes() == 1 {
            Ok(to_value(format!("{} minute ago", date.num_minutes())).unwrap())
        } else {
            Ok(to_value(format!("{} minutes ago", date.num_minutes())).unwrap())
        }
    } else {
        if date.num_seconds() == 1 {
            Ok(to_value(format!("{} second ago", date.num_seconds())).unwrap())
        } else {
            Ok(to_value(format!("{} seconds ago", date.num_seconds())).unwrap())
        }
    }
}

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");

    let server_url = format!("{host}:{port}");
    println!("Connecting to the Database url: {db_url}");
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let mut templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");

    templates.register_filter("format_date", format_date);

    let state = AppState { templates, conn };

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(edit_user).post(update_user))
        .route("/users/new", get(new_user))
        .route("/users/delete/:id", post(delete_user))
        .route("/users/logout", get(logout_user))
        .route("/users/add/group/:id", post(add_user_into_group))
        .route("/users/remove/group/:id", post(remove_user_from_group))
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/:id", get(edit_group).post(update_group))
        .route("/groups/new", get(new_group))
        .route("/groups/delete/:id", post(delete_group))
        .route("/groups/add/space/:id", post(add_group_into_space))
        .route("/groups/remove/space/:id", post(remove_group_from_space))
        .route("/spaces", get(list_spaces).post(create_space))
        .route("/spaces/:id", get(edit_space).post(update_space))
        .route("/spaces/new", get(new_space))
        .route("/spaces/delete/:id", post(delete_space))
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/:id", get(edit_post).post(update_post))
        .route("/posts/new", get(new_post))
        .route("/posts/delete/:id", post(delete_post))
        .route_layer(middleware::from_fn_with_state(state.clone(), guard))
        .route("/", get(index))
        .route("/users/login", get(login).post(login_user))
        .route("/spaces/:id/view", get(view_space))
        .route("/posts/:id/view", get(view_post))
        .route("/posts/:id/raw", get(raw_post))
        .route("/upload", post(upload))
        .route_layer(middleware::from_fn_with_state(state.clone(), no_guard))
        .nest_service(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            .handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let addr = SocketAddr::from_str(&server_url).unwrap();
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    templates: Tera,
    conn: DatabaseConnection,
}

#[derive(Deserialize)]
struct Params {
    page: Option<u64>,
    items_per_page: Option<u64>,
}

async fn guard<T>(
    state: State<AppState>,
    mut cookies: Cookies,
    mut request: Request<T>,
    next: middleware::Next<T>
) -> Result<Response, StatusCode> {
    if let Some(value) = get_token_cookie(&cookies) {
        let token = value.access_token;
        let user = QueryCore::find_user_by_token(&state.conn, token)
            .await
            .expect("Cannot find user");
        if user.is_some() {
            request.extensions_mut().insert(user.unwrap());
            return Ok(next.run(request).await)
        }
    }
    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Error".to_string(),
            message: "You need to login to access this page.".to_string(),
        }),
    };

    Ok(guard_response(&mut cookies, data))
}


async fn no_guard<T>(
    state: State<AppState>,
    cookies: Cookies,
    mut request: Request<T>,
    next: middleware::Next<T>
) -> Result<Response, StatusCode> {
    let empty_user = UserModel {
        user_id: 0,
        user_name: "Anonymous".to_string(),
        user_password: "".to_string(),
        user_token: "".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Some(value) = get_token_cookie(&cookies) {
        let token = value.access_token;
        let user = QueryCore::find_user_by_token(&state.conn, token)
            .await
            .expect("Cannot find user");
        if user.is_none() {
            request.extensions_mut().insert(empty_user);
            return Ok(next.run(request).await)
        }
        request.extensions_mut().insert(user.unwrap());
        return Ok(next.run(request).await)
    }
    request.extensions_mut().insert(empty_user);
    return Ok(next.run(request).await)
}

async fn index() -> Redirect {
    Redirect::to("/spaces/1/view")
}

async fn list_users(
    state: State<AppState>,
    Query(params): Query<Params>,
    cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let users_per_page = params.items_per_page.unwrap_or(5);

    let (users, num_pages) = QueryCore::find_users_in_page(&state.conn, page, users_per_page)
        .await
        .expect("Cannot find users in page");

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("users", &users);
    ctx.insert("page", &page);
    ctx.insert("users_per_page", &users_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("users/index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn new_user(
    state: State<AppState>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);

    let body = state
        .templates
        .render("users/new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn edit_user(
    state: State<AppState>,
    mut cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
    Path(id): Path<i32>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();

    if logged_in_user.user_id != id && logged_in_user.user_name != "admin" {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to edit this user.".to_string(),
            }),
        };

        add_cookies(&mut cookies, data);

        ctx.insert("logged_in_user", &logged_in_user);

        if let Some(value) = get_flash_cookie(&cookies) {
            ctx.insert("flash", &value);
        }

        let body = state
            .templates
            .render("users/edit.html.tera", &ctx)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

        return Ok(Html(body))
    }

    let user: users::Model = QueryCore::find_user_by_id(&state.conn, id)
        .await
        .expect("could not find user")
        .unwrap_or_else(|| panic!("could not find user with id {id}"));

    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("user", &user);

    let body = state
        .templates
        .render("users/edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_user(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
    form: Form<users::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
    if logged_in_user.user_id != id && logged_in_user.user_name != "admin" {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_owned(),
                message: "You are not allowed to edit this user.".to_owned(),
            }),
        };
        let path = "/users".to_string();
        return Ok(post_response(&mut cookies, data, path))
    }

    let form = form.0;

    MutationCore::update_user_by_id(&state.conn, id, form)
        .await
        .expect("could not edit user");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "User successfully updated".to_owned(),
        }),
    };
    let path = "/users".to_string();
    Ok(post_response(&mut cookies, data, path))
}

async fn delete_user(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_user(&state.conn, id)
        .await
        .expect("could not delete post");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "User successfully deleted".to_owned(),
        }),
    };
    let path = "/users".to_string();
    Ok(post_response(&mut cookies, data, path))
}

async fn remove_user_from_group(
    state: State<AppState>,
    mut cookies: Cookies,
    Path(id): Path<i32>,
    form: Form<groups_users::MembersForm>,
) -> Result<PostResponse, (StatusCode, String)> {
    let form = form.0;
    /// Create Vec<i32> from comma separated string
    let user_ids: Vec<i32> = form
        .user_ids
        .split(',')
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    MutationCore::remove_users_from_group(&state.conn, id, user_ids)
        .await
        .expect("could not remove users from group");

    let message = format!("Users {} successfully removed", form.user_ids);
    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message,
        })
    };
    let path = "/groups/";
    let new_path = format!("{}{}", path, id);

    Ok(post_response(&mut cookies, data, new_path))
}


async fn add_user_into_group(
    state: State<AppState>,
    mut cookies: Cookies,
    Path(id): Path<i32>,
    form: Form<groups_users::MembersForm>,
) -> Result<PostResponse, (StatusCode, String)> {
    let form = form.0;
    /// Create Vec<i32> from comma separated string
    let user_ids: Vec<i32> = form
        .user_ids
        .split(',')
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    MutationCore::add_users_into_group(&state.conn, id, user_ids)
        .await
        .expect("could not remove users from group");

    let message = format!("Users {} successfully added", form.user_ids);
    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message,
        })
    };
    let path = "/groups/";
    let new_path = format!("{}{}", path, id);

    Ok(post_response(&mut cookies, data, new_path))
}


async fn login(
    state: State<AppState>,
    mut cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("users/login.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn logout_user(
    state: State<AppState>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "User successfully logged out".to_owned(),
        }),
    };
    let path = "/";
    Ok(logout_response(&mut cookies, data, path))
}

async fn login_user(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<users::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    let user = QueryCore::find_user_by_name(&state.conn, form.user_name)
        .await
        .expect("Cannot find user by name");

    if let Some(user) = user {
        if !MutationCore::verify_password(form.user_password, &user.user_password) {
            return Err((StatusCode::UNAUTHORIZED, "Unauthorized"));
        }

        let mut now = Utc::now();
        let iat = now.timestamp() as usize;
        let duration = Duration::seconds(86400); // 24 hours;
        let now = now + duration; // 30 days
        let exp = now.timestamp() as usize;

        let claims = Claims {
            sub: user.user_name,
            company: "ACME".to_owned(),
            exp,
            iat,
        };

        // Create the authorization token
        let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

        MutationCore::update_user_token(&state.conn, user.user_id, &token)
            .await
            .expect("could not set token");

        let data = Data {
            token: Option::from(TokenData {
                access_token: token
            }),
            flash: None
        };
        let path = "/";
        Ok(login_response(&mut cookies, data, path))
    } else {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized"));
    }
}

async fn create_user(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<users::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_user(&state.conn, form)
        .await
        .expect("could not insert user");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "User successfully created".to_owned(),
        })
    };
    let path = "/users".to_string();
    Ok(post_response(&mut cookies, data, path))
}



async fn list_groups(
    state: State<AppState>,
    Query(params): Query<Params>,
    Extension(logged_in_user): Extension<UserModel>,
    cookies: Cookies,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let groups_per_page = params.items_per_page.unwrap_or(5);

    let (groups, num_pages) = QueryCore::find_groups_in_page(&state.conn, page, groups_per_page)
        .await
        .expect("Cannot find groups in page");

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("groups", &groups);
    ctx.insert("page", &page);
    ctx.insert("groups_per_page", &groups_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("groups/index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn new_group(
    state: State<AppState>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);

    let body = state
        .templates
        .render("groups/new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn edit_group(
    state: State<AppState>,
    mut cookies: Cookies,
    Query(params): Query<Params>,
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();

    let group: groups::Model = QueryCore::find_group_by_id(&state.conn, id)
        .await
        .expect("could not find group")
        .unwrap_or_else(|| panic!("could not find group with id {id}"));

    if group.owner_id != logged_in_user.user_id {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to edit this group.".to_string(),
            }),
        };

        add_cookies(&mut cookies, data);

        ctx.insert("logged_in_user", &logged_in_user);

        if let Some(value) = get_flash_cookie(&cookies) {
            ctx.insert("flash", &value);
        }

        let body = state
            .templates
            .render("groups/edit.html.tera", &ctx)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

        return Ok(Html(body))
    }

    let page = params.page.unwrap_or(1);
    let users_per_page = params.items_per_page.unwrap_or(5);


    let (users, num_pages) = QueryCore::find_group_users_in_page(&state.conn, id, page, users_per_page)
        .await
        .expect("Cannot find groups in page");

    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("group", &group);
    ctx.insert("users", &users);
    ctx.insert("page", &page);
    ctx.insert("users_per_page", &users_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("groups/edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_group(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
    form: Form<groups::Model>,
) -> Result<PostResponse, (StatusCode, String)> {

    let group: groups::Model = QueryCore::find_group_by_id(&state.conn, id)
        .await
        .expect("could not find group")
        .unwrap_or_else(|| panic!("could not find group with id {id}"));

    if group.owner_id != logged_in_user.user_id {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to edit this group.".to_string(),
            }),
        };

        let path = "/groups".to_string();
        return Ok(post_response(&mut cookies, data, path))
    }

    let form = form.0;

    MutationCore::update_group_by_id(&state.conn, id, form)
        .await
        .expect("could not edit group");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Group successfully updated".to_owned(),
        })
    };
    let path = "/groups".to_string();
    Ok(post_response(&mut cookies, data, path))
}

async fn delete_group(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_group(&state.conn, id)
        .await
        .expect("could not delete post");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Group successfully deleted".to_owned(),
        })
    };
    let path = "/groups".to_string();
    Ok(post_response(&mut cookies, data, path))
}


async fn remove_group_from_space(
    state: State<AppState>,
    mut cookies: Cookies,
    Path(id): Path<i32>,
    form: Form<groups_spaces::MembersForm>,
) -> Result<PostResponse, (StatusCode, String)> {
    let form = form.0;
    /// Create Vec<i32> from comma separated string
    let group_ids: Vec<i32> = form
        .group_ids
        .split(',')
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    MutationCore::remove_groups_from_space(&state.conn, id, group_ids)
        .await
        .expect("could not remove groups from space");

    let message = format!("Groups {} successfully removed", form.group_ids);
    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message,
        })
    };
    let path = "/spaces/";
    let new_path = format!("{}{}", path, id);

    Ok(post_response(&mut cookies, data, new_path))
}


async fn add_group_into_space(
    state: State<AppState>,
    mut cookies: Cookies,
    Path(id): Path<i32>,
    form: Form<groups_spaces::MembersForm>,
) -> Result<PostResponse, (StatusCode, String)> {
    let form = form.0;
    /// Create Vec<i32> from comma separated string
    let group_ids: Vec<i32> = form
        .group_ids
        .split(',')
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    MutationCore::add_groups_into_space(&state.conn, id, group_ids)
        .await
        .expect("could not remove users from group");

    let message = format!("Groups {} successfully added", form.group_ids);
    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message,
        })
    };
    let path = "/spaces/";
    let new_path = format!("{}{}", path, id);

    Ok(post_response(&mut cookies, data, new_path))
}


async fn create_group(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<groups::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_group(&state.conn, form)
        .await
        .expect("could not insert group");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Group successfully created".to_owned(),
        })
    };
    let path = "/groups".to_string();
    Ok(post_response(&mut cookies, data, path))
}


async fn list_spaces(
    state: State<AppState>,
    Query(params): Query<Params>,
    Extension(logged_in_user): Extension<UserModel>,
    cookies: Cookies,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let spaces_per_page = params.items_per_page.unwrap_or(5);

    let (spaces, num_pages) = QueryCore::find_spaces_in_page_owned_by_user(&state.conn, logged_in_user.user_id, page, spaces_per_page)
        .await
        .expect("Cannot find spaces in page");

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("spaces", &spaces);
    ctx.insert("page", &page);
    ctx.insert("spaces_per_page", &spaces_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("spaces/index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn new_space(
    state: State<AppState>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);

    let body = state
        .templates
        .render("spaces/new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn edit_space(
    state: State<AppState>,
    mut cookies: Cookies,
    Query(params): Query<Params>,
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();

    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {id}"));

    if space.owner_id != Some(logged_in_user.user_id) {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to view the page.".to_string(),
            }),
        };

        add_cookies(&mut cookies, data);

        ctx.insert("logged_in_user", &logged_in_user);

        if let Some(value) = get_flash_cookie(&cookies) {
            ctx.insert("flash", &value);
        }

        let body = state
            .templates
            .render("spaces/view.html.tera", &ctx)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

        return Ok(Html(body))
    }

    let page = params.page.unwrap_or(1);
    let groups_per_page = params.items_per_page.unwrap_or(5);

    let (groups, num_pages) = QueryCore::find_space_groups_in_page(&state.conn, id, page, groups_per_page)
        .await
        .expect("Cannot find groups in page");

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("space", &space);
    ctx.insert("groups", &groups);
    ctx.insert("page", &page);
    ctx.insert("groups_per_page", &groups_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("spaces/edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn view_space(
    state: State<AppState>,
    Path(id): Path<i32>,
    Query(params): Query<Params>,
    mut cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();

    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {id}"));

    let users: Vec<i32> = QueryCore::find_users_by_space_id(&state.conn, id)
        .await
        .expect("Cannot find user in space groups");

    if !space.is_public && space.owner_id != Some(logged_in_user.user_id) && !users.contains(&logged_in_user.user_id) {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to view the page.".to_string(),
            }),
        };

        add_cookies(&mut cookies, data);

        ctx.insert("logged_in_user", &logged_in_user);

        if let Some(value) = get_flash_cookie(&cookies) {
            ctx.insert("flash", &value);
        }

        let body = state
            .templates
            .render("spaces/view.html.tera", &ctx)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

        return Ok(Html(body))
    }

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.items_per_page.unwrap_or(5);

    let (posts, num_pages) = QueryCore::find_posts_in_space(&state.conn, id, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("space", &space);
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("spaces/view.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_space(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
    form: Form<spaces::Model>,
) -> Result<PostResponse, (StatusCode, String)> {

    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {id}"));

    if space.owner_id != Some(logged_in_user.user_id) {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to view the page.".to_string(),
            }),
        };

        let path = "/spaces".to_string();
        return Ok(post_response(&mut cookies, data, path))
    }

    let form = form.0;

    MutationCore::update_space_by_id(&state.conn, id, form)
        .await
        .expect("could not edit space");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Space successfully updated".to_owned(),
        })
    };
    let path = "/spaces".to_string();
    Ok(post_response(&mut cookies, data, path))
}

async fn delete_space(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_space(&state.conn, id)
        .await
        .expect("could not delete post");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Space successfully deleted".to_owned(),
        })
    };
    let path = "/spaces".to_string();
    Ok(post_response(&mut cookies, data, path))
}

async fn create_space(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<spaces::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_space(&state.conn, form)
        .await
        .expect("could not insert space");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Space successfully created".to_owned(),
        })
    };
    let path = "/spaces".to_string();
    Ok(post_response(&mut cookies, data, path))
}


async fn list_posts(
    state: State<AppState>,
    Query(params): Query<Params>,
    cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.items_per_page.unwrap_or(5);

    let (posts, num_pages) = QueryCore::find_posts_in_page_owned_by_user(&state.conn, logged_in_user.user_id, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("posts/index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn new_post(
    state: State<AppState>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);

    let body = state
        .templates
        .render("posts/new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn create_post(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<posts::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_post(&state.conn, form)
        .await
        .expect("could not insert post");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Post successfully created".to_owned(),
        })
    };
    let path = "/posts".to_string();
    Ok(post_response(&mut cookies, data, path))
}


async fn view_post(
    state: State<AppState>,
    mut cookies: Cookies,
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();

    let post: posts::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let space_id = post.space_id;
    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, post.space_id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {space_id}"));

    let users: Vec<i32> = QueryCore::find_users_by_space_id(&state.conn, space_id)
        .await
        .expect("Cannot find user in space groups");

    if (!space.is_public || !post.post_published) && space.owner_id != Some(logged_in_user.user_id) && !users.contains(&logged_in_user.user_id){
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to view the page.".to_string(),
            }),
        };

        add_cookies(&mut cookies, data);

        ctx.insert("logged_in_user", &logged_in_user);

        if let Some(value) = get_flash_cookie(&cookies) {
            ctx.insert("flash", &value);
        }

        let body = state
            .templates
            .render("posts/view.html.tera", &ctx)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

        return Ok(Html(body))
    }

    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("posts/view.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn upload(
    state: State<AppState>,
    mut cookies: Cookies,
    mut multipart: Multipart
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let id = Uuid::new_v4().to_string();
    let domain = env::var("DOMAIN").expect("DOMAIN is not set in .env file");

    let server_url = format!("{domain}");
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();

        let mut file = File::create(format!("api/static/uploads/images/{}", id)).await.unwrap();
        file.write_all(&data).await
            .expect("could not write file");
    }
    let file_name = format!("{}/static/uploads/images/{}", server_url, id);
    let json = json!({
        "url": file_name,
    }).to_string();

    Ok(Html(json))
}

async fn raw_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let post: posts::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let space_id = post.space_id;
    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, post.space_id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {space_id}"));

    let users: Vec<i32> = QueryCore::find_users_by_space_id(&state.conn, space_id)
        .await
        .expect("Cannot find user in space groups");

    if (!space.is_public || !post.post_published) && space.owner_id != Some(logged_in_user.user_id) && !users.contains(&logged_in_user.user_id) {
        return Err((StatusCode::FORBIDDEN, "You are not allowed to view the page."))
    }

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("posts/raw.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn edit_post(
    state: State<AppState>,
    mut cookies: Cookies,
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let mut ctx = tera::Context::new();

    let post: posts::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let space_id = post.space_id;
    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, post.space_id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {space_id}"));

    if !space.is_public && space.owner_id != Some(logged_in_user.user_id) {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to view the page.".to_string(),
            }),
        };

        add_cookies(&mut cookies, data);

        ctx.insert("logged_in_user", &logged_in_user);

        if let Some(value) = get_flash_cookie(&cookies) {
            ctx.insert("flash", &value);
        }

        let body = state
            .templates
            .render("posts/view.html.tera", &ctx)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

        return Ok(Html(body))
    }

    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("posts/edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
    form: Form<posts::Model>,
) -> Result<PostResponse, (StatusCode, String)> {

    let post: posts::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let space_id = post.space_id;
    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, post.space_id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {space_id}"));

    if space.owner_id != Some(logged_in_user.user_id) {
        let data = Data {
            token: None,
            flash: Option::from(FlashData {
                kind: "Error".to_string(),
                message: "You are not allowed to view the page.".to_string(),
            }),
        };

        let path = "/posts".to_string();
        return Ok(post_response(&mut cookies, data, path))
    }

    let form = form.0;

    MutationCore::update_post_by_id(&state.conn, id, form)
        .await
        .expect("could not edit post");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Post successfully updated".to_owned(),
        })
    };
    let path = "/posts".to_string();
    Ok(post_response(&mut cookies, data, path))
}

async fn delete_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_post(&state.conn, id)
        .await
        .expect("could not delete post");

    let data = Data {
        token: None,
        flash: Option::from(FlashData {
            kind: "Success".to_owned(),
            message: "Post successfully deleted".to_owned(),
        })
    };
    let path = "/posts".to_string();
    Ok(post_response(&mut cookies, data, path))
}


/// JWT
///
///

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
    where
        S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}


pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
