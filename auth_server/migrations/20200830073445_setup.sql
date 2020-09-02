CREATE TABLE `accounts` 
(
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT COMMENT 'Identifier',
  `username` varchar(32) NOT NULL DEFAULT '',
  `sha_pass_hash` varchar(40) NOT NULL DEFAULT '',
  `sessionkey` varchar(80) NOT NULL DEFAULT '',
  `v` varchar(64) NOT NULL DEFAULT '',
  `s` varchar(64) NOT NULL DEFAULT '',
  `token_key` varchar(100) NOT NULL DEFAULT '',
  `banned` tinyint NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

INSERT INTO accounts VALUES
(NULL, 'test', "3d0d99423e31fcc67a6745ec89d70d700344bc76", '', '', '', '', 0),
(NULL, 'banned', "5a31ea4791dcb33648008d0c5c260baaa37e2a9a", '', '', '', '', 1);

CREATE TABLE `realms` 
(
  `id` int(10) NOT NULL AUTO_INCREMENT,
  `name` varchar(50) NOT NULL DEFAULT '0',
  `realm_type` tinyint NOT NULL DEFAULT '0',
  `flags` tinyint NOT NULL DEFAULT '0',
  `ip` varchar(50) NOT NULL DEFAULT '0',
  `population` float NOT NULL DEFAULT '0',
  `timezone` tinyint NOT NULL DEFAULT '1',
  `online` tinyint NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`) 
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

INSERT INTO realms VALUES
(NULL, 'testrealm', 0, 0, "127.0.0.1:8085", 0.0, 1, 0)
