CREATE TABLE captchas (
    id            BIGINT  UNSIGNED NOT NULL AUTO_INCREMENT,
    answer        VARCHAR(6)       NOT NULL,
    expires       DATETIME         NOT NULL,
    PRIMARY KEY (id)
);