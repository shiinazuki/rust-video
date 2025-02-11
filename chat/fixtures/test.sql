-- insert 3 workspaces
INSERT INTO
    workspaces (name, owner_id)
VALUES
    ('acme', 0),
    ('foo', 0),
    ('bar', 0);

-- insert 5 users with hash password = 123456
INSERT INTO
    users (ws_id, email, fullname, password_hash)
VALUES
    (
        1,
        'test@acme.org',
        'test',
        '$argon2id$v=19$m=19456,t=2,p=1$tuf+xQpdHETIKpdAb078nQ$N8nDgQguI8uEw1IaE7MhmXtjg0p+BMGZx+83pfnjym4'
    ),
    (
        1,
        'tom@acme.org',
        'tom',
        '$argon2id$v=19$m=19456,t=2,p=1$tuf+xQpdHETIKpdAb078nQ$N8nDgQguI8uEw1IaE7MhmXtjg0p+BMGZx+83pfnjym4'
    ),
    (
        1,
        'marry@acme.org',
        'marry',
        '$argon2id$v=19$m=19456,t=2,p=1$tuf+xQpdHETIKpdAb078nQ$N8nDgQguI8uEw1IaE7MhmXtjg0p+BMGZx+83pfnjym4'
    ),
    (
        1,
        'jack@acme.org',
        'jack',
        '$argon2id$v=19$m=19456,t=2,p=1$tuf+xQpdHETIKpdAb078nQ$N8nDgQguI8uEw1IaE7MhmXtjg0p+BMGZx+83pfnjym4'
    ),
    (
        1,
        'jerry@acme.org',
        'jerry',
        '$argon2id$v=19$m=19456,t=2,p=1$tuf+xQpdHETIKpdAb078nQ$N8nDgQguI8uEw1IaE7MhmXtjg0p+BMGZx+83pfnjym4'
    );

-- insert 4 chat
-- insert public/private channel
INSERT INTO
    chats (ws_id, name, type, members)
VALUES
    (1, 'general', 'public_channel', '{1, 2, 3, 4, 5}'),
    (1, 'private', 'private_channel', '{1, 2, 3}');

-- insert unnamed chat
INSERT INTO
    chats (ws_id, type, members)
VALUES
    (1, 'single', '{1, 2}'),
    (1, 'group', '{1, 3, 4}');
