CREATE TABLE sessions (
    id          INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    user_id     INTEGER UNSIGNED NOT NULL,
    ip_address  VARCHAR(45)      NOT NULL,
    user_agent  VARCHAR(512)     NOT NULL,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at    DATETIME         NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);