CREATE TABLE board_groups (
    id              INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    group_name      TINYTEXT         NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE boards (
    id              INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    group_id        INTEGER UNSIGNED,
    handle          VARCHAR(8)       NOT NULL  UNIQUE,
    title           TINYTEXT         NOT NULL,
    access_level    TINYINT UNSIGNED NOT NULL,
    nsfw            boolean          NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (group_id) REFERENCES board_groups(id) ON DELETE CASCADE
);