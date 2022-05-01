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
	`o` float NOT NULL DEFAULT '0',
	`instance_id` int (10) unsigned NOT NULL DEFAULT '0',
	`bind_zone` smallint(5) unsigned NOT NULL DEFAULT '0',
	`bind_map` smallint(5) unsigned NOT NULL DEFAULT '0',
	`bind_x` float NOT NULL DEFAULT '0',
	`bind_y` float NOT NULL DEFAULT '0',
	`bind_z` float NOT NULL DEFAULT '0',
	`guild_id` int(10) unsigned NOT NULL DEFAULT '0',
	`tutorial_data` binary(8) NOT NULL DEFAULT '0',
	`playtime_total` int(10) unsigned NOT NULL DEFAULT '0',
	`playtime_level` int(10) unsigned NOT NULL DEFAULT '0',
	PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `character_account_data` (
	`character_id` int(10) unsigned NOT NULL DEFAULT '0',
	`data_type` tinyint unsigned NOT NULL DEFAULT '0',
	`time` bigint unsigned NOT NULL DEFAULT '0',
	`decompressed_size` int unsigned NOT NULL DEFAULT '0',
	`data` longblob,
	KEY `FK_CHARACTER_ACCOUNT_DATA_CHARACTER` (`character_id`),
	CONSTRAINT `FK_CHARACTER_ACCOUNT_DATA_CHARACTER` FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT,
	PRIMARY KEY (`character_id`, `data_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `playercreateinfo` (
	`race` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT 'A bit-mask corresponding to races that should get the spell. ',
	`class` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT 'A bit-mask corresponding to class that should get the spell.',
	`map` smallint(5) unsigned NOT NULL DEFAULT '0' COMMENT 'The map ID (See Map.dbc)',
	`zone` smallint(5) unsigned NOT NULL DEFAULT '0' COMMENT 'A zone identifier.',
	`position_x` float NOT NULL DEFAULT '0' COMMENT 'The X position for the characters initial position.',
	`position_y` float NOT NULL DEFAULT '0' COMMENT 'The Y position for the characters initial position.',
	`position_z` float NOT NULL DEFAULT '0' COMMENT 'The Z position for the characters initial position.',
	`orientation` float NOT NULL DEFAULT '0' COMMENT 'The orientation for the characters initial position.',
	PRIMARY KEY (`race`,`class`)
) ENGINE=MyISAM DEFAULT CHARSET=utf8;

CREATE TABLE `areatrigger_teleport` (
  `id` int(11) unsigned NOT NULL DEFAULT '0' COMMENT 'The ID of the trigger (See AreaTrigger.dbc).',
  `name` text DEFAULT NULL COMMENT 'The name of the teleport areatrigger.',
  `required_level` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT 'The player needs to be at least this level.',
  `required_item` int(11) unsigned NOT NULL DEFAULT '0' COMMENT 'Requested an item (See item_template.entry).',
  `required_item2` int(11) unsigned NOT NULL DEFAULT '0' COMMENT 'Requested an item (See item_template.entry).',
  `heroic_key` int(11) unsigned NOT NULL DEFAULT '0',
  `heroic_key2` int(11) unsigned NOT NULL DEFAULT '0',
  `required_quest_done` int(11) unsigned NOT NULL DEFAULT '0' COMMENT 'Requires quest (See quest_template.entry).',
  `required_quest_done_heroic` int(11) unsigned NOT NULL DEFAULT '0',
  `target_map` smallint(5) unsigned NOT NULL DEFAULT '0' COMMENT 'The destination map id. (See map.dbc)',
  `target_position_x` float NOT NULL DEFAULT '0' COMMENT 'The x location of the player at the destination.',
  `target_position_y` float NOT NULL DEFAULT '0' COMMENT 'The y location of the player at the destination.',
  `target_position_z` float NOT NULL DEFAULT '0' COMMENT 'The z location of the player at the destination.',
  `target_orientation` float NOT NULL DEFAULT '0' COMMENT 'The orientation of the player at the destination.',
  PRIMARY KEY (`id`),
  FULLTEXT KEY `name` (`name`)
) ENGINE=MyISAM DEFAULT CHARSET=utf8 ROW_FORMAT=DYNAMIC COMMENT='Trigger System';
