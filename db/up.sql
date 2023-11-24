-- Your SQL goes here
CREATE TABLE USERS (
    ID SERIAL NOT NULL,
    NAME VARCHAR(255) NOT NULL,
    EMAIL VARCHAR(255) NOT NULL,
    HASHED_PASSWORD VARCHAR(255) NOT NULL,
    SALT VARCHAR(255) NOT NULL,
    PUBLIC_KEY VARCHAR(255) NOT NULL,
    ADDRESS VARCHAR(255) NOT NULL,
    PRIMARY KEY (EMAIL)
)