CREATE TABLE  threads (
    id        INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    board     INTEGER UNSIGNED NOT NULL,
    title     TEXT,
    views     INTEGER UNSIGNED NOT NULL,
    sticky    BOOLEAN          NOT NULL,
    archived  BOOLEAN          NOT NULL,
    bump_time DATETIME         NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (board) REFERENCES boards(id) ON DELETE CASCADE
);

CREATE TABLE thread_ids (
    id         INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    user_id    INTEGER UNSIGNED NOT NULL,
    thread_id  INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (thread_id) REFERENCES threads(id) ON DELETE CASCADE
);