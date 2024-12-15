CREATE TABLE applications (
    id          INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    user_id     INTEGER UNSIGNED NOT NULL,
    accepted    BOOLEAN          NOT NULL,
    background  TEXT             NOT NULL,
    motivation  TEXT             NOT NULL,
    other       TEXT             NOT NULL,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closed_at   DATETIME,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE application_reviews (
    id              INTEGER UNSIGNED  NOT NULL  AUTO_INCREMENT,
    reviewer_id     INTEGER UNSIGNED  NOT NULL,
    application_id  INTEGER UNSIGNED  NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (reviewer_id) REFERENCES users(id),
    FOREIGN KEY (application_id) REFERENCES applications(id) ON DELETE CASCADE
);