CREATE TABLE users (
    id            INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    access_level  TINYINT UNSIGNED NOT NULL,
    username      VARCHAR(16)      NOT NULL UNIQUE,
    email         VARCHAR(128)     UNIQUE,
    password_hash VARCHAR(128)     NOT NULL,
    created_at    DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);