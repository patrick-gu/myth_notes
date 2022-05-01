use myth::{header, html, Responder, Response, StatusCode};
use sailfish::TemplateOnce;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::models::{self, user::SignupError};

pub(super) async fn get() -> myth::Result<Response> {
    let template = Template { error: None };
    let markup = template.render_once()?;
    Ok(html(markup))
}

#[derive(Deserialize)]
pub(super) struct RequestBody {
    username: String,
    password: String,
}

pub(super) async fn post(
    RequestBody { username, password }: RequestBody,
    pool: SqlitePool,
) -> myth::Result<Response> {
    let result = models::user::signup(&pool, username, password).await?;
    match result {
        Ok(session) => Ok(Response::default()
            .with_status(StatusCode::SEE_OTHER)
            .add_header(header::LOCATION, "/notes")
            .add_header(
                header::SET_COOKIE,
                format!(
                    "session={}; Path=/; Max-Age={}",
                    session,
                    models::user::SESSION_TIME - 1
                ),
            )),
        Err(error) => {
            let template = Template { error: Some(error) };
            let markup = template.render_once()?;
            Ok(html(markup).with_status(StatusCode::BAD_REQUEST))
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "signup.stpl")]
struct Template {
    error: Option<SignupError>,
}
