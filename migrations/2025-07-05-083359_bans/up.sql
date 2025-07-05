CREATE TABLE bans (
    id                 INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    moderator_id       BIGINT  UNSIGNED NOT NULL,
    user_id            BIGINT  UNSIGNED,
    post_id            INTEGER UNSIGNED,
    reason             TEXT,
    ip_address         VARCHAR(45)      NOT NULL,
    expires_at         DATETIME         NOT NULL,
    created_at         DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (moderator_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE SET NULL
);