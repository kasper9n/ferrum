create table tracks (
	id                TEXT PRIMARY KEY NOT NULL,
	filesize          INTEGER NOT NULL, -- i64
	duration_s        REAL NOT NULL, -- f64
	bitrate           REAL NOT NULL, -- f64
	sample_rate       REAL NOT NULL, -- f64
	file              TEXT NOT NULL,
	modified_at       INTEGER NOT NULL, -- i64 ms since unix epoch
	added_at          INTEGER NOT NULL, -- i64 ms since unix epoch
	name              TEXT NOT NULL,
	artist            TEXT NOT NULL,
	imported_from     TEXT NULL,
	original_id       TEXT NULL, -- Imported ID, like iTunes Persistent ID
	composer          TEXT NULL,
	sort_name         TEXT NULL,
	sort_artist       TEXT NULL,
	sort_composer     TEXT NULL,
	genre             TEXT NULL,
	rating_pct        INTEGER NULL, -- from 0 to 100
	year              INTEGER NULL, -- i64
	bpm               REAL NULL, -- f64
	comments          TEXT NULL,
	grouping          TEXT NULL,
	liked             BOOLEAN NULL,
	disliked          BOOLEAN NULL,
	disabled          BOOLEAN NULL,
	compilation       BOOLEAN NULL,
	album_name        TEXT NULL,
	album_artist      TEXT NULL,
	sort_album_name   TEXT NULL,
	sort_album_artist TEXT NULL,
	track_num         INTEGER NULL, -- u32
	track_count       INTEGER NULL, -- u32
	disc_num          INTEGER NULL, -- u32
	disc_count        INTEGER NULL, -- u32
	imported_at       INTEGER NULL,
	play_count        INTEGER NULL, -- u32
	skip_count        INTEGER NULL, -- u32
	volume            INTEGER NULL -- from -100 to 100
);

CREATE TABLE plays (
	date          INTEGER PRIMARY KEY NOT NULL,
	track_id      TEXT NOT NULL REFERENCES tracks(id)
);

CREATE TABLE plays_imported (
	date_range_from INTEGER PRIMARY KEY NOT NULL,
	date_range_to   INTEGER NOT NULL,
	count           INTEGER NOT NULL,
	track_id        TEXT NOT NULL REFERENCES tracks(id)
);

CREATE TABLE skips (
	date          INTEGER PRIMARY KEY NOT NULL,
	track_id      TEXT NOT NULL REFERENCES tracks(id)
);

CREATE TABLE skips_imported (
	date_range_from INTEGER PRIMARY KEY NOT NULL,
	date_range_to   INTEGER NOT NULL,
	count           INTEGER NOT NULL,
	track_id        TEXT NOT NULL REFERENCES tracks(id)
);

CREATE TABLE track_lists (
	id            TEXT PRIMARY KEY NOT NULL,
	type          TEXT NOT NULL CHECK (type IN ('playlist', 'folder', 'special')),
	parent_id     TEXT NULL REFERENCES track_lists(id),
	item_index    INTEGER NULL,
	name          TEXT NOT NULL,
	description   TEXT NOT NULL,
	liked         BOOLEAN NOT NULL DEFAULT 0,
	disliked      BOOLEAN NOT NULL DEFAULT 0,
	imported_from TEXT NULL, -- For example "itunes"
	original_id   TEXT NULL, -- For example iTunes Persistent ID
	imported_at   INTEGER NULL,
	created_at    INTEGER NULL -- Nullable for imported playlists
);

CREATE TABLE playlist_tracks (
	track_list_id TEXT NOT NULL REFERENCES track_lists(id),
	track_id      TEXT NOT NULL REFERENCES tracks(id),
	item_index    INTEGER NOT NULL,
	PRIMARY KEY (track_list_id, item_index)
);

CREATE TABLE play_times (
	started_at INTEGER PRIMARY KEY NOT NULL,
	duration   INTEGER NOT NULL,
	track_id   TEXT NOT NULL REFERENCES tracks(id)
);
