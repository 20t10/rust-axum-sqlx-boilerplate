create extension if not exists pgcrypto;

create table if not exists users (
    id uuid primary key,
    email text not null unique,
    password_hash text not null,
    role text not null,
    created_at timestamptz not null default now()
);
