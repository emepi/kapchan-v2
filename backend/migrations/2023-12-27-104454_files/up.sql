CREATE TABLE files (
    id           INTEGER UNSIGNED  NOT NULL  AUTO_INCREMENT,
    name         VARCHAR(64)       NOT NULL,
    hash         VARCHAR(32)       NOT NULL,
    type         SMALLINT UNSIGNED NOT NULL,
    size         INTEGER UNSIGNED  NOT NULL,
    location     VARCHAR(512)      NOT NULL,
    created_at   DATETIME          NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    uploaded_by  INTEGER UNSIGNED  NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (uploaded_by) REFERENCES users(id) ON DELETE CASCADE
);