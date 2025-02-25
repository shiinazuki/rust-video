-- Add migration script here
CREATE TYPE gender as ENUM(
    'female',
    'male',
    'unknown'
);

CREATE TABLE user_stats(
    email VARCHAR(128) NOT NULL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    gender gender DEFAULT 'unknown', 
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_visited_at TIMESTAMPTZ,
    last_watched_at TIMESTAMPTZ,
    recent_watched INT[],
    viewed_but_not_started INT[],
    started_but_not_finished INT[],
    finished INT[],
    last_email_notification TIMESTAMPTZ,
    last_in_app_notification TIMESTAMPTZ,
    last_sms_notification TIMESTAMPTZ
);

CREATE INDEX user_stats_created_at_idx ON user_stats(created_at);
CREATE INDEX user_stats_last_visited_at_idx ON user_stats(last_visited_at);
CREATE INDEX user_stats_last_watched_at_idx ON user_stats(last_watched_at);
CREATE INDEX user_stats_recent_watched_idx ON user_stats USING GIN(recent_watched);
CREATE INDEX user_stats_viewed_but_not_started_idx ON user_stats USING GIN(viewed_but_not_started);
CREATE INDEX user_stats_started_but_not_finished_idx ON user_stats USING GIN(started_but_not_finished);
CREATE INDEX user_stats_last_email_notification_idx ON user_stats(last_email_notification);
CREATE INDEX user_stats_last_in_app_notification_idx ON user_stats(last_in_app_notification);
CREATE INDEX user_stats_last_sms_notification_idx ON user_stats(last_sms_notification);
