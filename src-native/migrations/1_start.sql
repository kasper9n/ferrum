create table tracks (
	id                TEXT PRIMARY KEY,
	filesize          INTEGER NOT NULL, -- u64 (or i64 for js)
	duration_s        REAL NOT NULL, -- f64
	bitrate           REAL NOT NULL, -- f64
	sample_rate       REAL NOT NULL, -- f64
	file              TEXT NOT NULL, -- f64
	modified_at       INTEGER NOT NULL, -- u64 ms since unix epoch
	added_at          INTEGER NOT NULL, -- u64 ms since unix epoch
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
	id            TEXT PRIMARY KEY,
	track_id      TEXT NOT NULL REFERENCES tracks(id),
	date          INTEGER NULL,
	date_range_to INTEGER NULL,
	CHECK ((date IS NULL) != (date_range_to IS NULL))
);

CREATE TABLE skips (
	id            TEXT PRIMARY KEY,
	track_id      TEXt NOT NULL REFERENCES tracks(id),
	date          INTEGER NULL,
	date_range_to INTEGER NULL,
	CHECK ((date IS NULL) != (date_range_to IS NULL))
);

CREATE TABLE track_lists (
	id            TEXT PRIMARY KEY,
	type          TEXT NOT NULL CHECK (type IN ('playlist', 'folder', 'special')),
	parent_id     TEXT NULL REFERENCES track_lists(id),
	position      INTEGER NULL,
	name          TEXT NOT NULL,
	description   TEXT NOT NULL,
	liked         BOOLEAN NOT NULL DEFAULT 0,
	disliked      BOOLEAN NOT NULL DEFAULT 0,
	imported_from TEXT NULL, -- For example "itunes"
	original_id   TEXT NULL, -- For example iTunes Persistent ID
	imported_at   INTEGER NULL,
	created_at    INTEGER NOT NULL
);

CREATE TABLE playlist_tracks (
	track_list_id TEXT NOT NULL REFERENCES track_lists(id),
	track_id      TEXT NOT NULL REFERENCES tracks(id),
	position      INTEGER NOT NULL,
	PRIMARY KEY (track_list_id, position)
);

CREATE TABLE play_times (
	id         TEXT PRIMARY KEY,
	track_id   TEXT NOT NULL REFERENCES tracks(id),
	started_at INTEGER NOT NULL,
	duration   INTEGER NOT NULL,
	-- v1 playtime has two issues:
	-- - some durations are double counted (or triple, etc.)
	-- - timestamps aren't updated after pausing
	is_v1      BOOLEAN NOT NULL DEFAULT 0
);
