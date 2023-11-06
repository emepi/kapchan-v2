CREATE TABLE boards (
    id                INTEGER           NOT NULL AUTO_INCREMENT,
    handle            VARCHAR(8)        UNIQUE NOT NULL,
    title             VARCHAR(64)       UNIQUE NOT NULL,
    full_description  VARCHAR(4096),
    read_access       TINYINT           NOT NULL,
    post_access       TINYINT           NOT NULL,
    attachments_limit TINYINT           NOT NULL,
    post_limit        SMALLINT UNSIGNED NOT NULL,
    geo_locations     BOOLEAN           NOT NULL,
    visible           BOOLEAN           NOT NULL,
    nsfw              BOOLEAN           NOT NULL,
    created_at        DATETIME          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE board_attachments (
    id           INTEGER NOT NULL AUTO_INCREMENT,
    board        INTEGER NOT NULL,
    allowed_type INTEGER NOT NULL, -- file format id
    size_limit   BIGINT  NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (board) REFERENCES boards(id) ON DELETE CASCADE
);

CREATE TABLE posts (
    id          INTEGER  NOT NULL AUTO_INCREMENT,
    board       INTEGER  NOT NULL,
    thread      INTEGER, -- OP is null
    owner       INTEGER  NOT NULL,
    title       TINYTEXT,
    content     TEXT,
    read_access TINYINT  NOT NULL,
    post_access TINYINT  NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (board)  REFERENCES boards(id)   ON DELETE CASCADE,
    FOREIGN KEY (thread) REFERENCES posts(id)    ON DELETE CASCADE,
    FOREIGN KEY (owner)  REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE post_attachments (
    id   INTEGER NOT NULL AUTO_INCREMENT,
    post INTEGER NOT NULL,
    file INTEGER NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (post) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (file) REFERENCES files(id) ON DELETE CASCADE
);