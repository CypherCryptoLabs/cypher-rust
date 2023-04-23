CREATE DATABASE IF NOT EXISTS cypher;
USE cypher;

CREATE TABLE IF NOT EXISTS blocks (
    id INT AUTO_INCREMENT PRIMARY KEY,
    timestamp BIGINT UNSIGNED NOT NULL,
    parent_block_hash CHAR(64) NOT NULL,
    payload JSON NOT NULL,
    forger_signature CHAR(140) NOT NULL,
    validators JSON NOT NULL
);

CREATE TABLE IF NOT EXISTS tx (
    id INT AUTO_INCREMENT PRIMARY KEY,
    amount BIGINT UNSIGNED NOT NULL,
    network_fee BIGINT UNSIGNED NOT NULL,
    sender_pub_key CHAR(130) NOT NULL,
    receiver_address VARCHAR(255) NOT NULL,
    timestamp BIGINT UNSIGNED NOT NULL,
    signature CHAR(140) NOT NULL
);