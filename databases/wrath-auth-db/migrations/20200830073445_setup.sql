CREATE TABLE `accounts` 
(
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT COMMENT 'Identifier',
  `username` varchar(32) NOT NULL DEFAULT '',
  `sha_pass_hash` varchar(40) NOT NULL DEFAULT '',
  `sessionkey` varchar(80) NOT NULL DEFAULT '',
  `v` varchar(64) NOT NULL DEFAULT '',
  `s` varchar(64) NOT NULL DEFAULT '',
  `token_key` varchar(100) NOT NULL DEFAULT '',
  `banned` tinyint(1) unsigned zerofill NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO accounts VALUES
(NULL, 'test', "3d0d99423e31fcc67a6745ec89d70d700344bc76", '', '', '', '', 0),
(NULL, 'banned', "5a31ea4791dcb33648008d0c5c260baaa37e2a9a", '', '', '', '', 1);

CREATE TABLE `realms` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(50) NOT NULL DEFAULT '',
  `realm_type` tinyint(1) unsigned zerofill NOT NULL DEFAULT '0',
  `flags` tinyint(1) unsigned zerofill NOT NULL DEFAULT '0',
  `ip` varchar(50) NOT NULL DEFAULT '0',
  `population` float NOT NULL DEFAULT '0',
  `timezone` tinyint(1) unsigned zerofill NOT NULL DEFAULT '1',
  `online` tinyint(1) unsigned zerofill NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO realms VALUES
(NULL, 'testrealm', 0, 0, "127.0.0.1:8085", 0.0, 1, 0);

CREATE TABLE `account_data` (
	`account_id` int unsigned NOT NULL DEFAULT '0',
	`data_type` int unsigned NOT NULL DEFAULT '0',
	`time` bigint unsigned NOT NULL DEFAULT '0',
	`decompressed_size` int unsigned NOT NULL default '0',
	`data` longblob,
	KEY `FK_ACCOUNT_DATA_ACCOUNT` (`account_id`),
	CONSTRAINT `FK_ACCOUNT` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `realm_characters` (
	`account_id` int unsigned NOT NULL DEFAULT '0',
	`realm_id` int unsigned NOT NULL DEFAULT '0',
	`num_characters` tinyint unsigned NOT NULL default '0',
	KEY `FK_REALM_CHARACTERS_ACCOUNT` (`account_id`),
	CONSTRAINT `FK_REALM_CHARACTERS_ACCOUNT` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT,
	KEY `FK_REALM_CHARACTERS_REALM` (`realm_id`),
	CONSTRAINT `FK_REALM_CHARACTERS_REALM` FOREIGN KEY (`realm_id`) REFERENCES `realms` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT,
	PRIMARY KEY (`account_id`, `realm_id`)
);

