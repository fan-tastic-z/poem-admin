-- Add migration script here
CREATE TABLE IF NOT EXISTS account (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL DEFAULT '',
    password TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    remark TEXT NOT NULL DEFAULT '',
    last_login_time TIMESTAMPTZ,
    is_deletable BOOLEAN NOT NULL DEFAULT FALSE,
    organization_id BIGINT NOT NULL DEFAULT 0,
    organization_name TEXT NOT NULL DEFAULT '',
    role_id BIGINT NOT NULL DEFAULT 0,
    role_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS role (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_by BIGINT NOT NULL DEFAULT 0,
    created_by_name TEXT NOT NULL DEFAULT '',
    is_deletable BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS menu (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    parent_id BIGINT NOT NULL DEFAULT 0,
    parent_name TEXT NOT NULL DEFAULT '',
    order_index BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS role_menu (
    id BIGSERIAL PRIMARY KEY,
    role_id BIGINT NOT NULL DEFAULT 0,
    role_name TEXT NOT NULL DEFAULT '',
    menu_id BIGINT NOT NULL DEFAULT 0,
    menu_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS organization (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    parent_id BIGINT NOT NULL DEFAULT 0,
    parent_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS route (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    method TEXT NOT NULL DEFAULT '',
    menu_id BIGINT NOT NULL DEFAULT 0,
    menu_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS operation_log (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL DEFAULT 0,
    account_name TEXT NOT NULL DEFAULT '',
    ip_address TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    operation_type TEXT NOT NULL DEFAULT '',
    operation_module TEXT NOT NULL DEFAULT '',
    operation_description TEXT NOT NULL DEFAULT '',
    operation_result TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_operation_log_account_id ON operation_log (account_id);
CREATE INDEX idx_operation_log_operation_type ON operation_log (operation_type);
CREATE INDEX idx_operation_log_operation_module ON operation_log (operation_module);
CREATE INDEX idx_operation_log_created_at ON operation_log (created_at);
