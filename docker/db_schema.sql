CREATE DATABASE IF NOT EXISTS cypher;
USE cypher;

CREATE TABLE IF NOT EXISTS block (
    id INT AUTO_INCREMENT PRIMARY KEY,
    timestamp BIGINT UNSIGNED NOT NULL,
    parent_block_hash VARCHAR(64) NOT NULL,
    forger VARCHAR(255) NOT NULL,
    forger_signature VARCHAR(255) NOT NULL,
    forger_pub_key VARCHAR(130) NOT NULL
);

CREATE TABLE IF NOT EXISTS tx (
    id INT AUTO_INCREMENT PRIMARY KEY,
    amount BIGINT UNSIGNED NOT NULL,
    network_fee BIGINT UNSIGNED NOT NULL,
    sender_pub_key VARCHAR(130) NOT NULL,
    receiver_address VARCHAR(255) NOT NULL,
    timestamp BIGINT UNSIGNED NOT NULL,
    signature VARCHAR(140) NOT NULL
);

CREATE TABLE IF NOT EXISTS vouch (
    id INT AUTO_INCREMENT PRIMARY KEY,
    address VARCHAR(255) NOT NULL,
    signature VARCHAR(140) NOT NULL,
    pub_key VARCHAR(130) NOT NULL
);