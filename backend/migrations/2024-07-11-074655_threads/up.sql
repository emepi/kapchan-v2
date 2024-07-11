CREATE TABLE threads (
    id              INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    board_id        INTEGER UNSIGNED,
    title           TINYTEXT         NOT NULL,
    pinned          BOOLEAN          NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
);