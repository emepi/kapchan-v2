CREATE TABLE threads (
    id           INTEGER UNSIGNED NOT NULL,
    board        INTEGER UNSIGNED NOT NULL,
    title        TINYTEXT         NOT NULL,
    pinned       BOOLEAN          NOT NULL,
    bump_date    DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    replies      INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (board) REFERENCES boards(id) ON DELETE CASCADE
)