CREATE TABLE users (
    id            INTEGER      NOT NULL AUTO_INCREMENT,
    access_level  TINYINT      NOT NULL,
    username      VARCHAR(16)  UNIQUE,
    email         VARCHAR(128) UNIQUE,
    password_hash VARCHAR(128),
    created_at    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE sessions (
    id          INTEGER     NOT NULL AUTO_INCREMENT,
    user_id     INTEGER     NOT NULL,
    mode        TINYINT     NOT NULL,
    ip_address  VARCHAR(45),
    user_agent  VARCHAR(512),
    created_at  DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at    DATETIME,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
)