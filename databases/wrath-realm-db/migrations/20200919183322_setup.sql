CREATE TABLE `characters` (
	`id` int(10) unsigned  NOT NULL AUTO_INCREMENT,
	`account_id` int(10) unsigned NOT NULL DEFAULT '0',
	`name` varchar(25) NOT NULL DEFAULT '',
	`race` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`class` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`gender` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`skin_color` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`face` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`hair_style` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`hair_color` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`facial_style` tinyint(3) unsigned NOT NULL DEFAULT '0',
	`player_flags` int(10) unsigned NOT NULL DEFAULT '0',
	`at_login_flags` smallint(5) unsigned NOT NULL DEFAULT '0',
	`zone` smallint(5) unsigned NOT NULL DEFAULT '0',
	`level` tinyint(3) unsigned NOT NULL DEFAULT '1',
	`map` smallint(5) unsigned NOT NULL DEFAULT '0',
	`x` float NOT NULL DEFAULT '0',
	`y` float NOT NULL DEFAULT '0',
	`z` float NOT NULL DEFAULT '0',
	`guild_id` int(10) unsigned NOT NULL DEFAULT '0',
	PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
