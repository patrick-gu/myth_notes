use myth::{errors::FilterError, header, Responder, Response, StatusCode};
use sqlx::{query, SqlitePool};

use crate::utils::unix_timestamp;

pub(crate) async fn authenticate(pool: &SqlitePool, session: &str) -> myth::Result<String> {
    let now = unix_timestamp();
    let option = query!(
        "select user_id from sessions where value = ? and expires_at > ?;",
        session,
        now
    )
    .fetch_optional(pool)
    .await?;
    let record = option.ok_or(Unauthorized)?;
    Ok(record.user_id)
}

#[derive(Debug)]
pub(crate) struct Unauthorized;

impl FilterError for Unauthorized {
    fn into_response(self: Box<Self>) -> myth::Response {
        Response::default()
            .with_status(StatusCode::SEE_OTHER)
            .add_header(header::SET_COOKIE, "session=expired; Path=/; Max-Age=0")
            .with_header(header::LOCATION, "/login")
    }
}

pub(crate) async fn logout(pool: &SqlitePool, session: &str) -> myth::Result<()> {
    let now = unix_timestamp();
    query!(
        "update sessions set expires_at = ? where value = ?;",
        now,
        session
    )
    .execute(pool)
    .await?;
    Ok(())
}
