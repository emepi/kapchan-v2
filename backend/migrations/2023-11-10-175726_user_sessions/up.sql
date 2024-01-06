CREATE TABLE sessions (
    id           INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    user_id      INTEGER UNSIGNED,
    access_level TINYINT UNSIGNED NOT NULL,
    mode         TINYINT UNSIGNED NOT NULL,
    ip_address   VARCHAR(45),
    user_agent   VARCHAR(512),
    created_at   DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at     DATETIME,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
)