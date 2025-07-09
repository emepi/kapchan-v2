CREATE TABLE chat_rooms (
    id            INTEGER UNSIGNED NOT NULL  AUTO_INCREMENT,
    name          VARCHAR(255)     NOT NULL,
    access_level  TINYINT UNSIGNED NOT NULL,
    PRIMARY KEY (id)
);