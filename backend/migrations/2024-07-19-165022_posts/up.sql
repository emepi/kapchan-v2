CREATE TABLE posts (
    id           INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    op_id        INTEGER UNSIGNED,
    body         TEXT             NOT NULL,
    access_level TINYINT UNSIGNED NOT NULL,
    created_at   DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (op_id) REFERENCES posts(id)
);

CREATE TABLE files (
    id           INTEGER UNSIGNED NOT NULL,
    file_name    TINYTEXT         NOT NULL,
    file_type    TINYTEXT         NOT NULL,
    thumbnail    VARCHAR(512)     NOT NULL,
    file_path    VARCHAR(512)     NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (id) REFERENCES posts(id) ON DELETE CASCADE
)