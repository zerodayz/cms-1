use axum::{http::{header, HeaderMap, HeaderValue, StatusCode}, response::Response};
use axum::response::{IntoResponse, Redirect};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use cookie::Expiration;
use cookie::time::{Duration, OffsetDateTime};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlashData {
    pub kind: String,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenData {
    pub access_token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Data {
    pub token: Option<TokenData>,
    pub flash: Option<FlashData>,
}


const AXUM_SESSION_NAME: &str = "_data";
const FLASH_MESSAGE_NAME: &str = "_flash";


pub fn guard_response(cookies: &mut Cookies, data: Data) -> Response
{
    let mut data = data;
    let mut cookie = Cookie::new(
        FLASH_MESSAGE_NAME,
        serde_json::to_string(&data.flash).unwrap(),
    );
    cookie.set_path("/");
    cookie.expires();
    let mut now = OffsetDateTime::now_utc();
    now += Duration::seconds(5);
    cookie.set_expires(now);
    cookies.add(cookie);
    let response = Redirect::permanent("/users/login")
        .into_response();
    response
}

pub type PostResponse = (StatusCode, HeaderMap);

pub fn post_response(cookies: &mut Cookies, data: Data, path: &'static str) -> PostResponse
{
    let mut data = data;
    let mut cookie = Cookie::new(
        FLASH_MESSAGE_NAME,
        serde_json::to_string(&data.flash).unwrap(),
    );
    cookie.set_path("/");
    cookie.expires();
    let mut now = OffsetDateTime::now_utc();
    now += Duration::seconds(5);
    cookie.set_expires(now);
    cookies.add(cookie);

    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static(path));

    (StatusCode::SEE_OTHER, header)
}

pub fn get_token_cookie(cookies: &Cookies) -> Option<TokenData>
{
    let session_cookie = cookies.get(AXUM_SESSION_NAME);
    if let Some(session_cookie) = session_cookie {
        let data = serde_json::from_str::<TokenData>(session_cookie.value());
        if let Ok(data) = data {
            Some(data)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_flash_cookie(cookies: &Cookies) -> Option<FlashData> {
    let session_cookie = cookies.get(FLASH_MESSAGE_NAME);
    if let Some(session_cookie) = session_cookie {
        let data = serde_json::from_str::<FlashData>(session_cookie.value());
        if let Ok(data) = data {
            Some(data)
        } else {
            None
        }
    } else {
        None
    }
}

pub type LoginResponse = (StatusCode, HeaderMap);

pub fn login_response(cookies: &mut Cookies, data: Data, path: &'static str) -> LoginResponse
{
    let mut data = data;
    let mut cookie = Cookie::new(
        AXUM_SESSION_NAME,
        serde_json::to_string(&data.token).unwrap(),
    );

    cookie.set_path("/");
    cookie.set_http_only(true);
    cookies.add(cookie);

    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static(path));

    (StatusCode::SEE_OTHER, header)
}


pub type LogoutResponse = (StatusCode, HeaderMap);

pub fn logout_response<T>(cookies: &mut Cookies, data: T, path: &'static str) -> LogoutResponse
    where
        T: Serialize,
{
    cookies.remove(Cookie::named(AXUM_SESSION_NAME));

    let mut header = HeaderMap::new();
    header.insert(header::LOCATION, HeaderValue::from_static(path));

    (StatusCode::SEE_OTHER, header)
}
