CREATE TABLE board_groups (
    id    INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    name  TINYTEXT         NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE boards (
    id              INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    board_group_id  INTEGER UNSIGNED NOT NULL,
    handle          VARCHAR(8)       NOT NULL  UNIQUE,
    title           TINYTEXT         NOT NULL,
    description     TEXT,
    PRIMARY KEY (id),
    FOREIGN KEY (board_group_id) REFERENCES board_groups(id)
);