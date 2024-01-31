CREATE TABLE boards (
    id              INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    handle          VARCHAR(8)       NOT NULL  UNIQUE,
    title           TINYTEXT         NOT NULL,
    access_level    TINYINT UNSIGNED NOT NULL,
    bump_limit      INTEGER UNSIGNED NOT NULL, 
    nsfw            boolean          NOT NULL,
    PRIMARY KEY (id)
);