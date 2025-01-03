CREATE TABLE boards (
    id                     INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    handle                 VARCHAR(8)       NOT NULL  UNIQUE,
    title                  TINYTEXT         NOT NULL,
    description            TEXT             NOT NULL,
    access_level           TINYINT UNSIGNED NOT NULL,
    active_threads_limit   INTEGER UNSIGNED NOT NULL,
    thread_size_limit      INTEGER UNSIGNED NOT NULL,
    unique_posts           boolean          NOT NULL,
    captcha                boolean          NOT NULL,
    nsfw                   boolean          NOT NULL,
    PRIMARY KEY (id)
);