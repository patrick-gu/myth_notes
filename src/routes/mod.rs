mod account;
mod index;
mod login;
mod notes;
mod signup;

use myth::{
    errors::FilterError,
    header::{self, HeaderValue},
    impl_Filter, Filter, Responder, Response, StatusCode,
};
use sqlx::SqlitePool;

use crate::models;

pub(crate) fn filter(pool: impl_Filter!(SqlitePool => Clone)) -> impl_Filter!(Response) {
    let index_filter = myth::path::end()
        .and(myth::method::get())
        .and(not_logged_in())
        .handle(index::get);
    let signup_filter = myth::path::literal("signup")
        .and(myth::path::end())
        .then(
            (myth::method::get().and(not_logged_in()).handle(signup::get)).or(myth::method::post()
                .and(myth::form::urlencoded::request())
                .and(pool.clone())
                .handle(signup::post)),
        )
        .dynamic();
    let login_filter = myth::path::literal("login")
        .and(myth::path::end())
        .then(
            (myth::method::get().and(not_logged_in()).handle(login::get)).or(myth::method::post()
                .and(myth::form::urlencoded::request())
                .and(pool.clone())
                .handle(login::post)),
        )
        .dynamic();
    let notes_filter = myth::path::literal("notes")
        .then(
            (myth::path::end()
                .then(
                    (myth::method::get()
                        .and(authenticate(pool.clone()))
                        .and(pool.clone())
                        .handle(notes::get)
                        .dynamic())
                    .or(myth::method::post()
                        .and(authenticate(pool.clone()))
                        .and(pool.clone())
                        .handle(notes::post)
                        .dynamic()),
                )
                .dynamic())
            .or(myth::path::param::<String>()
                .then(
                    (myth::path::end().then(
                        (myth::method::get()
                            .and(authenticate(pool.clone()))
                            .and(pool.clone())
                            .receive::<(String,)>()
                            .handle(notes::get_one))
                        .or(myth::method::post()
                            .and(myth::form::urlencoded::request())
                            .and(authenticate(pool.clone()))
                            .and(pool.clone())
                            .receive::<(String,)>()
                            .handle(notes::post_one)
                            .dynamic()),
                    ))
                    .or(myth::path::literal("delete")
                        .and(myth::path::end())
                        .and(myth::method::post())
                        .and(authenticate(pool.clone()))
                        .and(pool.clone())
                        .receive::<(String,)>()
                        .handle(notes::delete_one)
                        .dynamic()),
                )
                .dynamic()),
        )
        .dynamic();
    let account_filter = myth::path::literal("account")
        .then(
            (myth::path::end()
                .and(myth::method::get())
                .and(authenticate(pool.clone()))
                .and(pool.clone())
                .handle(account::get))
            .or(myth::path::literal("logout")
                .then(
                    (myth::path::end()
                        .and(myth::method::post())
                        .and(authenticate_and_session(pool.clone()))
                        .and(pool.clone())
                        .handle(account::logout))
                    .or(myth::path::literal("all")
                        .and(myth::path::end())
                        .and(myth::method::post())
                        .and(authenticate(pool.clone()))
                        .and(pool.clone())
                        .handle(account::logout_all)),
                )
                .dynamic())
            .or(myth::path::literal("password")
                .and(myth::path::end())
                .and(myth::method::post())
                .and(myth::form::urlencoded::request())
                .and(authenticate(pool.clone()))
                .and(pool.clone())
                .handle(account::update_password)
                .dynamic())
            .or(myth::path::literal("delete")
                .and(myth::path::end())
                .and(myth::method::post())
                .and(myth::form::urlencoded::request())
                .and(authenticate(pool.clone()))
                .and(pool)
                .handle(account::delete)
                .dynamic()),
        )
        .dynamic();
    index_filter
        .or(signup_filter)
        .or(login_filter)
        .or(notes_filter)
        .or(account_filter)
}

fn not_logged_in() -> impl_Filter!(()) {
    #[derive(Debug)]
    struct IsLoggedIn;
    impl FilterError for IsLoggedIn {
        fn into_response(self: Box<Self>) -> Response {
            Response::default()
                .with_status(StatusCode::SEE_OTHER)
                .with_header(header::LOCATION, "/notes")
        }
    }

    async fn handler(session: Option<&str>) -> myth::Result<()> {
        if session.is_some() {
            Err(IsLoggedIn.into())
        } else {
            Ok(())
        }
    }

    get_session().handle(handler).untuple()
}

fn authenticate(pool: impl_Filter!(SqlitePool)) -> impl_Filter!(String) {
    async fn handler(session: Option<&str>, pool: SqlitePool) -> myth::Result<String> {
        if let Some(session) = session {
            models::session::authenticate(&pool, session).await
        } else {
            Err(NotLoggedIn.into())
        }
    }
    get_session().and(pool).handle(handler)
}

fn authenticate_and_session(pool: impl_Filter!(SqlitePool)) -> impl_Filter!('f, (String, &'f str)) {
    async fn handler(session: Option<&str>, pool: SqlitePool) -> myth::Result<(String, &str)> {
        if let Some(session) = session {
            Ok((
                models::session::authenticate(&pool, session).await?,
                session,
            ))
        } else {
            Err(NotLoggedIn.into())
        }
    }
    get_session().and(pool).handle(handler).untuple()
}

#[derive(Debug)]
struct NotLoggedIn;
impl FilterError for NotLoggedIn {
    fn into_response(self: Box<Self>) -> Response {
        Response::default()
            .with_status(StatusCode::SEE_OTHER)
            .with_header(header::LOCATION, "/login")
    }
}

fn get_session() -> impl_Filter!('f, Option<&'f str>) {
    async fn handler(cookie_value: Option<&HeaderValue>) -> myth::Result<Option<&str>> {
        let cookie_value = match cookie_value {
            Some(value) => value,
            None => return Ok(None),
        };

        #[derive(Debug)]
        struct BadCookie;
        impl FilterError for BadCookie {
            fn into_response(self: Box<Self>) -> Response {
                Response::default().with_status(StatusCode::BAD_REQUEST)
            }
        }

        let value = cookie_value.to_str().map_err(|_| BadCookie)?;
        let mut session = None;
        for s in value.split("; ") {
            if let Some((key, value)) = s.split_once('=') {
                if key == "session" {
                    if session.is_some() {
                        return Err(BadCookie.into());
                    }
                    session = Some(value);
                }
            } else {
                return Err(BadCookie.into());
            }
        }
        Ok(session)
    }
    header::value_optional(header::COOKIE).handle(handler)
}
