mod models;
mod routes;
mod utils;

pub use models::run_migrations;
use myth::{impl_Filter, Response};
use sqlx::SqlitePool;

pub fn filter(pool: SqlitePool) -> impl_Filter!(Response) {
    let pool = myth::cloning(pool);
    routes::filter(pool)
}
