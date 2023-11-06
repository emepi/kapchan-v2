CREATE TABLE files (
    id          INTEGER  NOT NULL AUTO_INCREMENT,
    type        INTEGER  NOT NULL, -- id for file format
    file_size   BIGINT   NOT NULL, -- in bytes
    owner       INTEGER  NOT NULL,
    md5_hash    CHAR(32) UNIQUE NOT NULL,
    location    VARCHAR(512), -- omitted if file is archived internally
    file_name   VARCHAR(32),
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (owner) REFERENCES sessions(id) ON DELETE CASCADE
);

-- offload files to database server (not recommended for performance)
CREATE TABLE archives (
    id   INTEGER  NOT NULL AUTO_INCREMENT,
    file INTEGER  NOT NULL,
    data LONGBLOB NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (file) REFERENCES files(id) ON DELETE CASCADE
);