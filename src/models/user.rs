use std::borrow::Cow;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{error::DatabaseError, query, sqlite::SqliteError, SqlitePool};

use crate::utils::{create_uuid, unix_timestamp};

pub(crate) const SESSION_TIME: i64 = 7 * 24 * 60 * 60;

pub(crate) async fn signup(
    pool: &SqlitePool,
    username: String,
    password: String,
) -> myth::Result<Result<String, SignupError>> {
    if !is_valid_username(&username) {
        return Ok(Err(SignupError::BadUsername));
    }
    if !is_strong_password(&password) {
        return Ok(Err(SignupError::BadPassword));
    }

    let (id, note_id, session, salt) = {
        let mut rng = thread_rng();
        let id = create_uuid(&mut rng);
        let note_id = create_uuid(&mut rng);
        let session: String = (&mut rng)
            .sample_iter(Alphanumeric)
            .map(char::from)
            .take(50)
            .collect();
        let salt = SaltString::generate(&mut rng);
        (id, note_id, session, salt)
    };
    let password_hash = tokio::task::spawn_blocking(move || -> myth::Result<String> {
        let password_hash = argon2().hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    })
    .await??;

    let now = unix_timestamp();
    let expires_at = now.checked_add(SESSION_TIME).unwrap();

    let result = query!(
        "insert into users (id, username, password_hash) values (?, ?, ?); insert into sessions (value, user_id, expires_at) values (?, ?, ?); insert into notes (id, user_id, title, data, updated_at) values (?, ?, 'Welcome to Myth Notes', 'This is an example note.', ?);",
        id,
        username,
        password_hash,
        session,
        id,
        expires_at,
        note_id,
        id,now
    )
    .execute(pool)
    .await;
    if let Err(error) = result {
        if let Some(error) = error.as_database_error() {
            if let Some(error) = error.try_downcast_ref::<SqliteError>() {
                if error.code() == Some(Cow::Borrowed("2067"))
                    && error.message() == "UNIQUE constraint failed: users.username"
                {
                    return Ok(Err(SignupError::UsernameAlreadyExists));
                }
            }
        }
        return Err(error.into());
    }

    Ok(Ok(session))
}

pub(crate) enum SignupError {
    BadUsername,
    BadPassword,
    UsernameAlreadyExists,
}

pub(crate) async fn login(
    pool: &SqlitePool,
    username: String,
    password: String,
) -> myth::Result<Result<String, LoginError>> {
    let session: String = thread_rng()
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(50)
        .collect();
    let option = query!(
        "select id, password_hash from users where username = ?;",
        username
    )
    .fetch_optional(pool)
    .await?;
    let record = match option {
        Some(record) => record,
        None => return Ok(Err(LoginError::UsernameNotFound)),
    };
    let id = record.id;
    let password_hash: String = record.password_hash;
    let password_matches = tokio::task::spawn_blocking(move || -> myth::Result<bool> {
        let password_hash = PasswordHash::new(&password_hash)?;
        Ok(argon2()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok())
    })
    .await??;
    if !password_matches {
        return Ok(Err(LoginError::WrongPassword));
    }

    let expires_at = unix_timestamp().checked_add(SESSION_TIME).unwrap();

    query!(
        "insert into sessions (value, user_id, expires_at) values (?, ?, ?);",
        session,
        id,
        expires_at,
    )
    .execute(pool)
    .await?;

    Ok(Ok(session))
}

pub(crate) enum LoginError {
    UsernameNotFound,
    WrongPassword,
}

fn argon2() -> Argon2<'static> {
    Argon2::default()
}

pub(crate) async fn username(pool: &SqlitePool, id: &str) -> myth::Result<String> {
    Ok(query!("select username from users where id = ?;", id)
        .fetch_one(pool)
        .await?
        .username)
}

pub(crate) async fn logout_all(pool: &SqlitePool, id: &str) -> myth::Result<()> {
    let now = unix_timestamp();
    query!(
        "update sessions set expires_at = min(expires_at, ?) where user_id = ?;",
        now,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn update_password(
    pool: &SqlitePool,
    id: &str,
    old_password: String,
    new_password: String,
) -> myth::Result<Result<(), UpdatePasswordError>> {
    if old_password == new_password {
        return Ok(Err(UpdatePasswordError::Same));
    }
    if !is_strong_password(&new_password) {
        return Ok(Err(UpdatePasswordError::BadNew));
    }

    let record = query!("select password_hash from users where id = ?;", id)
        .fetch_one(pool)
        .await?;
    let old_password_hash: String = record.password_hash;
    let old_password_matches = tokio::task::spawn_blocking(move || -> myth::Result<bool> {
        let old_password_hash = PasswordHash::new(&old_password_hash)?;
        Ok(argon2()
            .verify_password(old_password.as_bytes(), &old_password_hash)
            .is_ok())
    })
    .await??;
    if !old_password_matches {
        return Ok(Err(UpdatePasswordError::WrongCurrent));
    }

    let salt = SaltString::generate(thread_rng());

    let new_password_hash = tokio::task::spawn_blocking(move || -> myth::Result<String> {
        let new_password_hash = argon2().hash_password(new_password.as_bytes(), &salt)?;
        Ok(new_password_hash.to_string())
    })
    .await??;

    query!(
        "update users set password_hash = ? where id = ?;",
        new_password_hash,
        id
    )
    .execute(pool)
    .await?;

    Ok(Ok(()))
}

pub(crate) enum UpdatePasswordError {
    BadNew,
    WrongCurrent,
    Same,
}

pub(crate) async fn delete_account(
    pool: &SqlitePool,
    id: &str,
    password: String,
) -> myth::Result<Result<(), DeleteAccountError>> {
    let record = query!("select password_hash from users where id = ?;", id)
        .fetch_one(pool)
        .await?;
    let password_hash: String = record.password_hash;
    let password_matches = tokio::task::spawn_blocking(move || -> myth::Result<bool> {
        let password_hash = PasswordHash::new(&password_hash)?;
        Ok(argon2()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok())
    })
    .await??;
    if !password_matches {
        return Ok(Err(DeleteAccountError::WrongPassword));
    }

    query!("delete from users where id = ?;", id)
        .execute(pool)
        .await?;
    Ok(Ok(()))
}

pub(crate) enum DeleteAccountError {
    WrongPassword,
}

fn is_valid_username(username: &str) -> bool {
    (3..=20).contains(&username.len()) && username.chars().all(|c| c.is_ascii_alphanumeric())
}

fn is_strong_password(password: &str) -> bool {
    password.len() >= 10
}
