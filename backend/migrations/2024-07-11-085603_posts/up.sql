CREATE TABLE posts (
    id                 INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    session_id         INTEGER UNSIGNED NOT NULL,
    thread_id          INTEGER UNSIGNED NOT NULL,
    access_level       TINYINT UNSIGNED NOT NULL,
    tripcode           VARCHAR(10)      NOT NULL,
    message            TEXT,
    created_at         DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (thread_id) REFERENCES threads(id)
);

CREATE TABLE replies (
    post_id         INTEGER UNSIGNED NOT NULL,
    reply_id        INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (post_id, reply_id),
    FOREIGN KEY (post_id)  REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (reply_id) REFERENCES posts(id) ON DELETE CASCADE
);

CREATE TABLE attachments (
    id                 INTEGER UNSIGNED NOT NULL,
    file_name          TINYTEXT,
    file_location      VARCHAR(512),
    thumbnail_location VARCHAR(512),
    PRIMARY KEY (id),
    FOREIGN KEY (id) REFERENCES posts(id)
);