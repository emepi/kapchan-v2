CREATE TABLE board_groups (
    id    INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    name  TINYTEXT         NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE boards (
    id           INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    board_group  INTEGER UNSIGNED,
    handle       VARCHAR(8)       NOT NULL  UNIQUE,
    title        TINYTEXT         NOT NULL,
    description  TEXT,
    created_at   DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by   INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (board_group) REFERENCES board_groups(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE board_flags (
    id       INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    board_id INTEGER UNSIGNED NOT NULL,
    flag     TINYINT UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
);