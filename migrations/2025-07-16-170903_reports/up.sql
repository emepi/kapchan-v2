CREATE TABLE reports (
    id            INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    post_id       INTEGER UNSIGNED NOT NULL,
    reason        TEXT             NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
);