-- Add migration script here

-- user table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    ws_id bigint NOT NULL,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces (
    id bigserial PRIMARY KEY,
    name varchar(32) NOT NULL UNIQUE,
    owner_id bigint NOT NULL REFERENCES users (id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

BEGIN;

-- add super user 0 and workspace 0
INSERT INTO
    users (id, ws_id, fullname, email, password_hash)
VALUES
    (0, 0, 'super user', 'super@none.org', '');
INSERT INTO
    workspaces (id, name, owner_id)
VALUES
    (0, 'none', 0);
COMMIT;

-- add foreign key constraint for ws_id for users
ALTER TABLE users
    ADD CONSTRAINT users_ws_id_fk
    FOREIGN KEY(ws_id)
    REFERENCES workspaces(id);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

-- chat type table
CREATE TYPE chat_type
AS ENUM ('single', 'group', 'private_channel', 'public_channel');


-- chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    ws_id bigint NOT NULL REFERENCES workspaces(id),
    name VARCHAR(64),
    type chat_type NOT NULL,
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


-- message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    files TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);

CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);
