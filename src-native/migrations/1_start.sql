create table tracks (
	id                INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
	text_id           TEXT UNIQUE NOT NULL,
	filesize          INTEGER NOT NULL, -- i64
	duration_s        REAL NOT NULL, -- f64
	bitrate           REAL NOT NULL, -- f64
	sample_rate       REAL NOT NULL, -- f64
	file              TEXT NOT NULL,
	modified_at       INTEGER NOT NULL, -- i64 ms since unix epoch
	added_at          INTEGER NOT NULL, -- i64 ms since unix epoch
	title             TEXT NOT NULL,
	artist            TEXT NOT NULL,
	imported_from     TEXT NULL,
	original_id       TEXT NULL, -- Imported ID, like iTunes Persistent ID
	composer          TEXT NULL,
	sort_title        TEXT NULL,
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
	album_title       TEXT NULL,
	album_artist      TEXT NULL,
	sort_album_title  TEXT NULL,
	sort_album_artist TEXT NULL,
	track_num         INTEGER NULL, -- u32
	track_count       INTEGER NULL, -- u32
	disc_num          INTEGER NULL, -- u32
	disc_count        INTEGER NULL, -- u32
	imported_at       INTEGER NULL,
	play_count        INTEGER NOT NULL, -- u32
	skip_count        INTEGER NOT NULL, -- u32
	volume            INTEGER NULL -- from -100 to 100
);

CREATE TRIGGER search_queue_after_track_insert AFTER INSERT ON tracks BEGIN
	INSERT OR IGNORE INTO search_queue (track_id) VALUES (NEW.id);
END;

CREATE TRIGGER search_queue_after_track_update AFTER UPDATE OF
	title,
	artist,
	album_title,
	album_artist,
	comments,
	genre,
	composer,
	grouping
ON tracks BEGIN
	INSERT OR IGNORE INTO search_queue (track_id) VALUES (NEW.id);
END;

CREATE TRIGGER search_queue_after_track_delete AFTER DELETE ON tracks BEGIN
	DELETE FROM search_ngrams WHERE track_id = OLD.id;
	DELETE FROM search_queue WHERE track_id = OLD.id;
END;

CREATE TABLE search_ngrams (
	ngram         TEXT NOT NULL,
	-- field IDs are defined by the Fields enum in filter.rs
	field         INTEGER NOT NULL,
	is_normalised BOOLEAN NOT NULL,
	track_id      INTEGER NOT NULL REFERENCES tracks(id),
	-- `ngram` first, because we always filter for it.
	-- if we do filter for `is_normalised`, we're doing an exact search. exact searches often return few results anyway (for example when searching é), so we can put `field` first.
	PRIMARY KEY (ngram, field, is_normalised, track_id)
) WITHOUT ROWID;

-- needed to purge a track's ngrams cheaply on reindex/delete
CREATE INDEX search_ngrams_by_track ON search_ngrams (track_id, field);

CREATE TABLE search_queue (
	id       INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
	track_id INTEGER NOT NULL UNIQUE REFERENCES tracks(id)
);

CREATE TABLE plays (
	date          INTEGER NOT NULL,
	track_id      INTEGER NOT NULL REFERENCES tracks(id),
	-- iTunes imports had non-unique play timestamps for some tracks, so we allow
	-- non-unique timestamps. iTunes plays are in plays_imported, except the most recent play.
	PRIMARY KEY (date, track_id)
);

CREATE TABLE plays_imported (
	date_range_from INTEGER NOT NULL,
	date_range_to   INTEGER NOT NULL,
	count           INTEGER NOT NULL,
	track_id        INTEGER NOT NULL REFERENCES tracks(id),
	PRIMARY KEY (date_range_from, track_id)
);

CREATE TABLE skips (
	date          INTEGER NOT NULL,
	track_id      INTEGER NOT NULL REFERENCES tracks(id),
	-- iTunes imports had duplicate play timestamps for some tracks, so we allow
	-- duplicate timestamps. iTunes plays are in plays_imported, except the most recent play.
	PRIMARY KEY (date, track_id)
);

CREATE TABLE skips_imported (
	date_range_from INTEGER NOT NULL,
	date_range_to   INTEGER NOT NULL,
	count           INTEGER NOT NULL,
	track_id        INTEGER NOT NULL REFERENCES tracks(id),
	PRIMARY KEY (date_range_from, track_id)
);

CREATE TABLE track_lists (
	id            TEXT PRIMARY KEY NOT NULL,
	kind          TEXT NOT NULL CHECK (kind IN ('playlist', 'folder', 'special')),
	parent_id     TEXT NULL REFERENCES track_lists(id),
	item_pos      INTEGER NULL,
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
	track_id      INTEGER NOT NULL REFERENCES tracks(id),
	item_id       INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
	item_pos      INTEGER NOT NULL
);

CREATE TABLE play_times (
	started_at INTEGER PRIMARY KEY NOT NULL,
	duration   INTEGER NOT NULL,
	track_id   INTEGER NOT NULL REFERENCES tracks(id)
);
