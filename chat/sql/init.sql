-- user table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    password VARCHAR(64) NOT NULL
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- chat type table
CREATE TABLE IF NOT EXISTS chat_type
AS ENUM ('single', 'group', 'private_channel', 'public_channel');


-- chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    type chat_type NOT NULL,
    members BIGINT[] NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


-- message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    image TEXT[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (chat_id) PEFERENCES chats(id),
    FOREIGN KEY (sender_id) PEFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_ad DESC);


CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id);
