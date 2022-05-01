# Myth Notes

A note taking app built with the [Myth](https://github.com/patrick-gu/myth) web framework.

This uses:

-   [Tokio](https://tokio.rs/) as the async runtime
-   [Sailfish](https://github.com/rust-sailfish/sailfish/) templates
-   [SQLx](https://github.com/launchbadge/sqlx) for access to an [SQLite](https://sqlite.org/index.html) database

## Usage

This uses [Rust](https://www.rust-lang.org/) 1.56.1.

The [SQLx CLI](https://github.com/launchbadge/sqlx/blob/master/sqlx-cli/README.md) is needed, and can be installed using:

```
cargo install sqlx-cli
```

### Preparing the database

```
mkdir data
export DATABASE_URL="sqlite:./data/notes.sqlite"
sqlx db create
sqlx migrate run
```

This will create and initialize the database.

#### Offline queries

Use the SQLx CLI to save query infomation into [sqlx-data.json](./sqlx-data.json) using:

```
cargo sqlx prepare -- --lib
```

### Running

For development:

```
cargo r
```

For production:

```
cargo r --release
```
