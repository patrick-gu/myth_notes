use myth_notes::{filter, run_migrations};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("myth=trace".parse().unwrap()),
        )
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NEW)
        .init();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:./data/notes.sqlite")
        .await
        .expect("failed to establish pool");

    run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let addr = if cfg!(debug_assertions) {
        ([127, 0, 0, 1], 8080)
    } else {
        ([0, 0, 0, 0], 80)
    };

    myth::serve(filter(pool)).bind(addr).run().await;
}
