CREATE TABLE `characters` (
	`id` int(10) unsigned zerofill NOT NULL AUTO_INCREMENT,
	`account_id` int(10) unsigned zerofill NOT NULL DEFAULT '0',
	`name` varchar(25) NOT NULL DEFAULT '',
	`race` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`class` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`gender` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`skin_color` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`face` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`hair_style` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`hair_color` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`facial_style` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`player_flags` int(10) unsigned zerofill NOT NULL DEFAULT '0000000000',
	`at_login_flags` smallint(5) unsigned zerofill NOT NULL DEFAULT '00000',
	`zone` smallint(5) unsigned zerofill NOT NULL DEFAULT '00000',
	`level` tinyint(3) unsigned zerofill NOT NULL DEFAULT '000',
	`map` smallint(5) unsigned zerofill NOT NULL DEFAULT '00000',
	`x` float NOT NULL DEFAULT '0',
	`y` float NOT NULL DEFAULT '0',
	`z` float NOT NULL DEFAULT '0',
	`guild_id` int(10) unsigned zerofill NOT NULL DEFAULT '0000000000',
	PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
