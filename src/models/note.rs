use rand::thread_rng;
use serde::Deserialize;
use sqlx::{query, query_as, SqlitePool};

use crate::utils::{create_uuid, unix_timestamp};

pub(crate) async fn list(pool: &SqlitePool, user_id: &str) -> myth::Result<Vec<NotePreview>> {
    let notes = query_as!(
        NotePreview,
        "select id, title from notes where user_id = ? order by updated_at desc;",
        user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(notes)
}

pub(crate) struct NotePreview {
    pub(crate) id: String,
    pub(crate) title: String,
}

pub(crate) async fn create(pool: &SqlitePool, user_id: &str) -> myth::Result<String> {
    let id = create_uuid(&mut thread_rng());
    let now = unix_timestamp();
    query!(
        "insert into notes (id, user_id, updated_at) values (?, ?, ?);",
        id,
        user_id,
        now,
    )
    .execute(pool)
    .await?;
    Ok(id)
}

pub(crate) async fn fetch(
    pool: &SqlitePool,
    user_id: &str,
    id: &str,
) -> myth::Result<Option<Note>> {
    Ok(query_as!(
        Note,
        "select title, data from notes where id = ? and user_id = ?;",
        id,
        user_id
    )
    .fetch_optional(pool)
    .await?)
}

pub(crate) async fn update(
    pool: &SqlitePool,
    user_id: &str,
    id: &str,
    note: &Note,
) -> myth::Result<()> {
    let now = unix_timestamp();
    query!(
        "update notes set title = ?, data = ?, updated_at = ? where id = ? and user_id = ?;",
        note.title,
        note.data,
        now,
        id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn delete(pool: &SqlitePool, user_id: &str, id: &str) -> myth::Result<()> {
    query!(
        "delete from notes where id = ? and user_id = ?;",
        id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Deserialize, Debug)]
pub(crate) struct Note {
    pub(crate) title: String,
    pub(crate) data: String,
}
