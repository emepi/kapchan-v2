CREATE TABLE threads (
    id              INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    board_id        INTEGER UNSIGNED NOT NULL,
    title           TINYTEXT         NOT NULL,
    pinned          BOOLEAN          NOT NULL,
    archived        BOOLEAN          NOT NULL,
    bump_time       DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
);