CREATE TABLE posts (
    id                 INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    user_id            BIGINT  UNSIGNED NOT NULL,
    thread_id          INTEGER UNSIGNED NOT NULL,
    show_username      BOOLEAN          NOT NULL,
    message            TEXT             NOT NULL,
    message_hash       VARCHAR(64)      NOT NULL,
    ip_address         VARCHAR(45)      NOT NULL,
    user_agent         VARCHAR(512)     NOT NULL,
    country_code       VARCHAR(2),
    hidden             BOOLEAN          NOT NULL,
    created_at         DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (thread_id) REFERENCES threads(id) ON DELETE CASCADE
);

CREATE TABLE replies (
    post_id            INTEGER UNSIGNED NOT NULL,
    reply_id           INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (post_id, reply_id),
    FOREIGN KEY (post_id)  REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (reply_id) REFERENCES posts(id) ON DELETE CASCADE
);

CREATE TABLE attachments (
    id                 INTEGER UNSIGNED NOT NULL,
    file_name          TINYTEXT         NOT NULL,
    file_type          TINYTEXT         NOT NULL,
    file_location      VARCHAR(512)     NOT NULL,
    thumbnail_location VARCHAR(512)     NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (id) REFERENCES posts(id) ON DELETE CASCADE
);