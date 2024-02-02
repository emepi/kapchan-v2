CREATE TABLE posts (
    id           INTEGER UNSIGNED  NOT NULL  AUTO_INCREMENT,
    user_id      INTEGER UNSIGNED  NOT NULL,
    thread       INTEGER UNSIGNED  NOT NULL,
    access_level TINYINT UNSIGNED  NOT NULL,
    ip_address   VARCHAR(45)       NOT NULL,
    attachment   INTEGER UNSIGNED,
    body         TEXT,
    sage         BOOLEAN           NOT NULL,
    note         TINYTEXT,
    created_at   DATETIME          NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (attachment) REFERENCES files(id),
    FOREIGN KEY (thread) REFERENCES threads(id)
);