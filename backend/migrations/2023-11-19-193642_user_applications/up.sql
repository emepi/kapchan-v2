CREATE TABLE applications (
    id          INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    user_id     INTEGER UNSIGNED NOT NULL,
    reviewer_id INTEGER UNSIGNED,
    referer_id  INTEGER UNSIGNED,
    accepted    BOOLEAN          NOT NULL,
    background  TEXT             NOT NULL,
    motivation  TEXT             NOT NULL,
    other       TEXT,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (reviewer_id) REFERENCES users(id),
    FOREIGN KEY (referer_id) REFERENCES users(id)
);