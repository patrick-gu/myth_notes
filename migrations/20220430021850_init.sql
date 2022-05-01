pragma foreign_keys = on;

create table users (
    id text primary key not null,
    username text unique not null,
    password_hash text not null
);

create table sessions (
    value text primary key not null,
    user_id text not null references users(id) on delete cascade,
    expires_at integer not null
);

create table notes (
    id text primary key not null,
    user_id id text not null references users(id) on delete cascade,
    title text not null default '',
    data text not null default '',
    updated_at integer not null
);
