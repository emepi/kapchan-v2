CREATE TABLE files (
    id           INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    hash         VARCHAR(32)      NOT NULL  UNIQUE,
    name         VARCHAR(200)     NOT NULL,
    location     VARCHAR(500)     NOT NULL,
    created_at   DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    uploaded_by  INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (uploaded_by) REFERENCES users(id)
);