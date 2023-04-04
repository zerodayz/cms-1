mod flash;

use axum::{async_trait, extract::{FromRequestParts, Form, Path, Query, State, TypedHeader}, headers::{authorization::Bearer, Authorization}, http::{request::Parts, Request, StatusCode}, response::{IntoResponse, Response, Html}, routing::{get, get_service, post}, Json, RequestPartsExt, Router, Server, middleware, Extension, RequestExt};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use csm_core::{
    sea_orm::{Database, DatabaseConnection},
    Mutation as MutationCore, Query as QueryCore,
};
use entity::users::Model as UserModel;
use entity::*;
use flash::{guard_response, get_flash_cookie, post_response, PostResponse, get_token_cookie, login_response, LoginResponse, logout_response, LogoutResponse, Data, FlashData, TokenData};
use migration::{Migrator, MigratorTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use std::{fmt::Display, env, net::SocketAddr};
use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::Redirect;
use chrono::{Duration, Utc};
use tera::Tera;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::services::ServeDir;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

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

    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");

    let state = AppState { templates, conn };

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/logout", get(logout_user))
        .route("/users/:id", get(edit_user).post(update_user))
        .route("/users/new", get(new_user))
        .route("/users/delete/:id", post(delete_user))
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/:id", get(edit_group).post(update_group))
        .route("/groups/new", get(new_group))
        .route("/groups/delete/:id", post(delete_group))
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
        .route_layer(middleware::from_fn_with_state(state.clone(), guard_no_fail))
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


async fn guard_no_fail<T>(
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
    Extension(logged_in_user): Extension<UserModel>,
    Path(id): Path<i32>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let user: users::Model = QueryCore::find_user_by_id(&state.conn, id)
        .await
        .expect("could not find user")
        .unwrap_or_else(|| panic!("could not find user with id {id}"));

    let mut ctx = tera::Context::new();
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
    form: Form<users::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
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
    let path = "/users";
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
    let path = "/users";
    Ok(post_response(&mut cookies, data, path))
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
    let path = "/users";
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
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let group: groups::Model = QueryCore::find_group_by_id(&state.conn, id)
        .await
        .expect("could not find group")
        .unwrap_or_else(|| panic!("could not find group with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("group", &group);

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
    form: Form<groups::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
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
    let path = "/groups";
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
    let path = "/groups";
    Ok(post_response(&mut cookies, data, path))
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
    let path = "/groups";
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

    let (spaces, num_pages) = QueryCore::find_spaces_in_page(&state.conn, page, spaces_per_page)
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
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("space", &space);

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
    cookies: Cookies,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let space: spaces::Model = QueryCore::find_space_by_id(&state.conn, id)
        .await
        .expect("could not find space")
        .unwrap_or_else(|| panic!("could not find space with id {id}"));

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.items_per_page.unwrap_or(5);

    let (posts, num_pages) = QueryCore::find_posts_in_space(&state.conn, id, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let mut ctx = tera::Context::new();
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
    form: Form<spaces::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
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
    let path = "/spaces";
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
    let path = "/spaces";
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
    let path = "/spaces";
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

    let (posts, num_pages) = QueryCore::find_posts_in_page(&state.conn, page, posts_per_page)
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
    let path = "/posts";
    Ok(post_response(&mut cookies, data, path))
}


async fn view_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let post: posts::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("logged_in_user", &logged_in_user);
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("posts/view.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
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
    Path(id): Path<i32>,
    Extension(logged_in_user): Extension<UserModel>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let post: posts::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let mut ctx = tera::Context::new();
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
    form: Form<posts::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
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
    let path = "/posts";
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
    let path = "/posts";
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