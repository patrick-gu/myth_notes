pub(crate) mod note;
pub(crate) mod session;
pub(crate) mod user;

use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> sqlx::Result<()> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}
