CREATE TABLE threads (
    id              INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    user_id         BIGINT  UNSIGNED NOT NULL,
    board_id        INTEGER UNSIGNED NOT NULL,
    title           TINYTEXT         NOT NULL,
    pinned          BOOLEAN          NOT NULL,
    archived        BOOLEAN          NOT NULL,
    bump_time       DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
);