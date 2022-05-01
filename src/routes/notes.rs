use myth::{header, html, Responder, Response, StatusCode};
use sailfish::TemplateOnce;
use sqlx::SqlitePool;

use crate::models::{
    self,
    note::{Note, NotePreview},
};

pub(super) async fn get(user_id: String, pool: SqlitePool) -> myth::Result<Response> {
    let notes = models::note::list(&pool, &user_id).await?;
    let template = NotesTemplate { notes: &notes };
    let markup = template.render_once()?;
    Ok(html(markup))
}

pub(super) async fn post(user_id: String, pool: SqlitePool) -> myth::Result<Response> {
    let id = models::note::create(&pool, &user_id).await?;
    Ok(Response::default()
        .with_status(StatusCode::SEE_OTHER)
        .with_header(header::LOCATION, format!("/notes/{}", id)))
}

#[derive(TemplateOnce)]
#[template(path = "notes.stpl")]
struct NotesTemplate<'a> {
    notes: &'a [NotePreview],
}

pub(super) async fn get_one(
    id: String,
    user_id: String,
    pool: SqlitePool,
) -> myth::Result<Response> {
    let note = models::note::fetch(&pool, &user_id, &id).await?;
    match note {
        Some(note) => {
            let template = NoteTemplate {
                id: &id,
                note: &note,
            };
            let markup = template.render_once()?;
            Ok(html(markup))
        }
        None => Ok(Response::default()
            .with_status(StatusCode::SEE_OTHER)
            .with_header(header::LOCATION, "/notes")),
    }
}

pub(super) async fn post_one(
    id: String,
    note: Note,
    user_id: String,
    pool: SqlitePool,
) -> myth::Result<Response> {
    models::note::update(&pool, &user_id, &id, &note).await?;
    let template = NoteTemplate {
        id: &id,
        note: &note,
    };
    let markup = template.render_once()?;
    Ok(html(markup))
}

pub(super) async fn delete_one(
    id: String,
    user_id: String,
    pool: SqlitePool,
) -> myth::Result<Response> {
    models::note::delete(&pool, &user_id, &id).await?;
    Ok(Response::default()
        .with_status(StatusCode::SEE_OTHER)
        .with_header(header::LOCATION, "/notes"))
}

#[derive(TemplateOnce)]
#[template(path = "note.stpl")]
struct NoteTemplate<'a> {
    id: &'a str,
    note: &'a Note,
}
