use myth::{header, html, Responder, Response, StatusCode};
use sailfish::TemplateOnce;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::models::{
    self,
    user::{DeleteAccountError, UpdatePasswordError},
};

pub(super) async fn get(id: String, pool: SqlitePool) -> myth::Result<Response> {
    let username = models::user::username(&pool, &id).await?;
    let template = Template {
        id: &id,
        username: &username,
        update_password_result: None,
        delete_account_error: None,
    };
    Ok(html(template.render_once()?))
}

pub(super) async fn logout(_id: String, session: &str, pool: SqlitePool) -> myth::Result<Response> {
    models::session::logout(&pool, session).await?;
    Ok(Response::default()
        .with_status(StatusCode::SEE_OTHER)
        .add_header(header::SET_COOKIE, "session=expired; Path=/; Max-Age=0")
        .with_header(header::LOCATION, "/"))
}

pub(super) async fn logout_all(id: String, pool: SqlitePool) -> myth::Result<Response> {
    models::user::logout_all(&pool, &id).await?;
    Ok(Response::default()
        .with_status(StatusCode::SEE_OTHER)
        .add_header(header::SET_COOKIE, "session=expired; Path=/; Max-Age=0")
        .with_header(header::LOCATION, "/"))
}

pub(super) async fn update_password(
    UpdatePassword {
        current_password,
        new_password,
    }: UpdatePassword,
    id: String,
    pool: SqlitePool,
) -> myth::Result<Response> {
    let result = models::user::update_password(&pool, &id, current_password, new_password).await?;
    let is_err = result.is_err();
    let username = models::user::username(&pool, &id).await?;
    let template = Template {
        id: &id,
        username: &username,
        update_password_result: Some(result),
        delete_account_error: None,
    };
    let mut response = html(template.render_once()?);
    if is_err {
        response = response.with_status(StatusCode::BAD_REQUEST);
    }
    Ok(response)
}

#[derive(Deserialize, Debug)]
pub(super) struct UpdatePassword {
    current_password: String,
    new_password: String,
}

pub(super) async fn delete(
    DeleteAccount { password }: DeleteAccount,
    id: String,
    pool: SqlitePool,
) -> myth::Result<Response> {
    let result = models::user::delete_account(&pool, &id, password).await?;
    match result {
        Ok(()) => Ok(Response::default()
            .with_status(StatusCode::SEE_OTHER)
            .add_header(header::SET_COOKIE, "session=expired; Path=/; Max-Age=0")
            .with_header(header::LOCATION, "/")),
        Err(error) => {
            let username = models::user::username(&pool, &id).await?;
            let template = Template {
                id: &id,
                username: &username,
                update_password_result: None,
                delete_account_error: Some(error),
            };
            Ok(html(template.render_once()?).with_status(StatusCode::BAD_REQUEST))
        }
    }
}

#[derive(Deserialize, Debug)]
pub(super) struct DeleteAccount {
    password: String,
}

#[derive(TemplateOnce)]
#[template(path = "account.stpl")]
struct Template<'a> {
    id: &'a str,
    username: &'a str,
    update_password_result: Option<Result<(), UpdatePasswordError>>,
    delete_account_error: Option<DeleteAccountError>,
}
