-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL DEFAULT '',
    password TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    remark TEXT NOT NULL DEFAULT '',
    last_login_time TIMESTAMPTZ NOT NULL,
    is_deleteable BOOLEAN NOT NULL DEFAULT FALSE,
    organization_id BIGINT NOT NULL DEFAULT 0,
    organization_name TEXT NOT NULL DEFAULT '',
    role_id BIGINT NOT NULL DEFAULT 0,
    role_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS role (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_by BIGINT NOT NULL DEFAULT 0,
    created_by_name TEXT NOT NULL DEFAULT '',
    is_deleteable BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS menu (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    parent_id BIGINT NOT NULL DEFAULT 0,
    parent_name TEXT NOT NULL DEFAULT '',
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS role_menu (
    id BIGINT PRIMARY KEY,
    role_id BIGINT NOT NULL DEFAULT 0,
    role_name TEXT NOT NULL DEFAULT '',
    menu_id BIGINT NOT NULL DEFAULT 0,
    menu_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS organization (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    parent_id BIGINT NOT NULL DEFAULT 0,
    parent_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS route (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    method TEXT NOT NULL DEFAULT '',
    menu_id BIGINT NOT NULL DEFAULT 0,
    menu_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

