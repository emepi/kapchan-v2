CREATE TABLE files (
    id                 INTEGER UNSIGNED  NOT NULL  AUTO_INCREMENT,
    user_id            INTEGER UNSIGNED  NOT NULL,
    access_level       TINYINT UNSIGNED  NOT NULL,
    ip_address         VARCHAR(45)       NOT NULL,
    name               VARCHAR(64)       NOT NULL,
    hash               VARCHAR(32)       NOT NULL UNIQUE,
    type               TINYINT UNSIGNED  NOT NULL,
    size               INTEGER UNSIGNED  NOT NULL,
    location           VARCHAR(512)      NOT NULL,
    thumbnail_location VARCHAR(512)      NOT NULL,
    created_at         DATETIME          NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);