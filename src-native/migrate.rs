use crate::library::Paths;
use crate::migrate::old_library::{Library, TrackList, TrackLists, load_library_json};
use anyhow::{Context, Result};
use sqlx::{
	ConnectOptions, Connection, Sqlite, migrate::MigrateDatabase, sqlite::SqliteConnectOptions,
};
use std::time::Instant;
use tempfile::TempDir;

pub async fn migrate_to_sqlite(paths: &Paths) -> Result<()> {
	let now = Instant::now();

	let library_json = match load_library_json(&paths.library_json)? {
		None => {
			return Ok(());
		}
		Some(library_json) => library_json,
	};

	let tmp_dir = TempDir::new().context("failed to create temp dir")?;
	let tmp_db = tmp_dir.path().join("Library.sqlite");

	Sqlite::create_database(&tmp_db.to_str().unwrap())
		.await
		.context("Could not create library database")?;
	let mut connection = SqliteConnectOptions::new()
		.filename(&tmp_db)
		.connect()
		.await
		.context("Error connecting to created library database")?;

	sqlx::migrate!("./src-native/migrations")
		.run_to(1, &mut connection)
		.await
		.context("Could not run database migrations")?;

	insert_library_into_db(&library_json, &mut connection)
		.await
		.context("Could not insert Library.json into database")?;

	connection
		.close()
		.await
		.context("Could not save/close database")?;

	std::fs::rename(&tmp_db, &paths.library_sqlite)
		.context("Failed to finalize sqlite database")?;

	println!("Migrated to SQLite: {}ms", now.elapsed().as_millis());

	Ok(())
}

async fn insert_library_into_db(
	library: &Library,
	conn: &mut sqlx::SqliteConnection,
) -> anyhow::Result<()> {
	let mut tx = conn.begin().await.context("Failed to begin transaction")?;
	// Defer foreign key checks for track list parent_id
	sqlx::query("PRAGMA defer_foreign_keys = ON;")
		.execute(&mut *tx)
		.await?;

	for (track_id, track) in &library.tracks {
		sqlx::query(
			"
				INSERT INTO tracks (
					id,
					filesize,
					duration_s,
					bitrate,
					sample_rate,
					file,
					modified_at,
					added_at,
					name,
					artist,
					imported_from,
					original_id,
					composer,
					sort_name,
					sort_artist,
					sort_composer,
					genre,
					rating_pct,
					year,
					bpm,
					comments,
					grouping,
					liked,
					disliked,
					disabled,
					compilation,
					album_name,
					album_artist,
					sort_album_name,
					sort_album_artist,
					track_num,
					track_count,
					disc_num,
					disc_count,
					imported_at,
					play_count,
					skip_count,
					volume
				) VALUES (
					?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
					?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
					?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
					?, ?, ?, ?, ?, ?, ?, ?
				)
			",
		)
		.bind(&track_id)
		.bind(track.size)
		.bind(track.duration)
		.bind(track.bitrate)
		.bind(track.sampleRate)
		.bind(&track.file)
		.bind(track.dateModified)
		.bind(track.dateAdded)
		.bind(&track.name)
		.bind(&track.artist)
		.bind(&track.importedFrom)
		.bind(&track.originalId)
		.bind(&track.composer)
		.bind(&track.sortName)
		.bind(&track.sortArtist)
		.bind(&track.sortComposer)
		.bind(&track.genre)
		.bind(track.rating)
		.bind(track.year)
		.bind(track.bpm)
		.bind(&track.comments)
		.bind(&track.grouping)
		.bind(track.liked)
		.bind(track.disliked)
		.bind(track.disabled)
		.bind(track.compilation)
		.bind(&track.albumName)
		.bind(&track.albumArtist)
		.bind(&track.sortAlbumName)
		.bind(&track.sortAlbumArtist)
		.bind(track.trackNum)
		.bind(track.trackCount)
		.bind(track.discNum)
		.bind(track.discCount)
		.bind(track.dateImported)
		.bind(track.playCount)
		.bind(track.skipCount)
		.bind(track.volume)
		.execute(&mut *tx)
		.await
		.with_context(|| format!("Failed to insert track {track_id}"))?;

		if let Some(plays) = &track.plays {
			for &date in plays {
				sqlx::query("INSERT INTO plays (date, track_id) VALUES (?, ?)")
					.bind(date)
					.bind(&track_id)
					.execute(&mut *tx)
					.await
					.with_context(|| format!("Failed to insert plays"))?;
			}
		}

		if let Some(imported) = &track.playsImported {
			for co in imported {
				sqlx::query(
					"INSERT INTO plays_imported (date_range_from, date_range_to, count, track_id) VALUES (?, ?, ?, ?)",
				)
				.bind(co.fromDate)
				.bind(co.toDate)
				.bind(co.count)
				.bind(&track_id)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert plays_imported"))?;
			}
		}

		if let Some(skips) = &track.skips {
			for &date in skips {
				sqlx::query("INSERT INTO skips (date, track_id) VALUES (?, ?)")
					.bind(date)
					.bind(&track_id)
					.execute(&mut *tx)
					.await
					.with_context(|| format!("Failed to insert skips"))?;
			}
		}

		if let Some(imported) = &track.skipsImported {
			for co in imported {
				sqlx::query(
					"INSERT INTO skips_imported (date_range_from, date_range_to, count, track_id) VALUES (?, ?, ?, ?)",
				)
				.bind(co.fromDate)
				.bind(co.toDate)
				.bind(co.count)
				.bind(&track_id)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert skips_imported"))?;
			}
		}
	}

	for (track_id, started_at, duration) in &library.v1PlayTime {
		sqlx::query(
			"INSERT INTO play_times (started_at, duration, track_id, is_v1) VALUES (?, ?, ?, 1)",
		)
		.bind(started_at)
		.bind(duration)
		.bind(track_id)
		.execute(&mut *tx)
		.await
		.with_context(|| format!("Failed to insert v1 play_times"))?;
	}
	for (track_id, started_at, duration) in &library.playTime {
		sqlx::query(
			"INSERT INTO play_times (started_at, duration, track_id, is_v1) VALUES (?, ?, ?, 1)",
		)
		.bind(started_at)
		.bind(duration)
		.bind(track_id)
		.execute(&mut *tx)
		.await
		.with_context(|| format!("Failed to insert play_times"))?;
	}

	let parent_map = build_parent_map(&library.trackLists);

	for (list_id, tracklist) in &library.trackLists {
		match tracklist {
			TrackList::Special(special) => {
				let name = special.name.to_string();
				sqlx::query(
					"
						INSERT INTO track_lists
							(id, type, name, description, created_at)
						VALUES (?, ?, ?, ?, ?)
					",
				)
				.bind(&special.id)
				.bind("folder")
				.bind(special.name.to_string())
				.bind("")
				.bind(special.dateCreated)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert special playlist {name}"))?;
			}
			TrackList::Folder(folder) => {
				let (index, parent_id) = parent_map
					.get(list_id.as_str())
					.with_context(|| format!("Parent of folder {} not found", folder.name))?;
				sqlx::query(
					"
						INSERT INTO track_lists
							(id, type, parent_id, item_index, name, description, liked, disliked,
							imported_from, original_id, imported_at, created_at)
						VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					",
				)
				.bind(&folder.id)
				.bind("folder")
				.bind(parent_id)
				.bind(index)
				.bind(&folder.name)
				.bind(folder.description.as_deref().unwrap_or(""))
				.bind(folder.liked)
				.bind(folder.disliked)
				.bind(&folder.importedFrom)
				.bind(&folder.originalId)
				.bind(folder.dateImported)
				.bind(folder.dateCreated)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert playlist folder {}", folder.name))?;
			}
			TrackList::Playlist(playlist) => {
				let (index, parent_id) = parent_map
					.get(list_id.as_str())
					.with_context(|| format!("Parent of playlist {} not found", playlist.name))?;
				sqlx::query(
					"
						INSERT INTO track_lists
							(id, type, parent_id, item_index, name, description, liked, disliked,
							imported_from, original_id, imported_at, created_at)
						VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					",
				)
				.bind(&playlist.id)
				.bind("folder")
				.bind(parent_id)
				.bind(index)
				.bind(&playlist.name)
				.bind(playlist.description.as_deref().unwrap_or(""))
				.bind(playlist.liked)
				.bind(playlist.disliked)
				.bind(&playlist.importedFrom)
				.bind(&playlist.originalId)
				.bind(playlist.dateImported)
				.bind(playlist.dateCreated)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert playlist {}", playlist.name))?;

				// playlist_tracks rows
				for (i, track_id) in playlist.tracks.iter().enumerate() {
					let i: i64 = i.try_into().unwrap();
					assert!(i >= 0);
					sqlx::query(
						"INSERT INTO playlist_tracks (track_list_id, track_id, item_index) VALUES (?, ?, ?)",
					)
					.bind(&playlist.id)
					.bind(track_id)
					.bind(i)
					.execute(&mut *tx)
					.await
					.with_context(|| {
						format!(
							"Failed to insert track {} in playlist {}",
							track_id, playlist.name
						)
					})?;
				}
			}
		}
	}

	tx.commit().await.context("Failed to commit transaction")?;
	Ok(())
}

/// Returns a map of playlist -> (index, parent_id) for every tracklist entry.
fn build_parent_map(track_lists: &TrackLists) -> std::collections::HashMap<String, (i64, String)> {
	let mut map = std::collections::HashMap::new();
	for (parent_id, tl) in track_lists {
		let children = match tl {
			TrackList::Folder(f) => &f.children,
			TrackList::Special(s) => &s.children,
			TrackList::Playlist(_) => continue,
		};
		for (i, child_id) in children.iter().enumerate() {
			// SQLite does not support u64
			let i: i64 = i.try_into().unwrap();
			assert!(i >= 0);
			map.insert(child_id.clone(), (i, parent_id.clone()));
		}
	}
	map
}

mod old_library {
	#![allow(non_snake_case)]

	use anyhow::{Context, Result};
	use linked_hash_map::LinkedHashMap;
	use serde::Deserialize;
	use serde_json::{Value, json};
	use std::fs::File;
	use std::io::{ErrorKind, Read, Seek, SeekFrom};

	pub fn load_library_json(library_json: &str) -> Result<Option<Library>> {
		let mut library_file = match File::open(&library_json) {
			Ok(file) => file,
			Err(err) => match err.kind() {
				ErrorKind::NotFound => return Ok(None),
				_ => return Err(err).context("Error opening library file"),
			},
		};

		let mut json_bytes = Vec::new();
		library_file
			.read_to_end(&mut json_bytes)
			.context("Error reading library file")?;

		let versioned_library: VersionedLibrary = match simd_json::from_slice(&mut json_bytes) {
			Ok(lib) => lib,
			Err(_) => {
				library_file
					.seek(SeekFrom::Start(0))
					.context("Error seeking to start of library file")?;
				let versioned_library = parse_old_versionless_library_json(&mut library_file)?;
				versioned_library
			}
		};

		let library = versioned_library.upgrade();
		Ok(Some(library))
	}

	fn parse_old_versionless_library_json(library_file: &mut File) -> Result<VersionedLibrary> {
		let mut json_str = String::new();
		library_file
			.read_to_string(&mut json_str)
			.context("Error reading library file")?;

		let mut value: Value =
			serde_json::from_str(&mut json_str).context("Error parsing library file")?;
		// Migrate version number to string
		if let Some(obj) = value.as_object_mut() {
			if let Some(version_field) = obj.get_mut("version") {
				if let Some(version) = version_field.as_number() {
					if version.as_u64() == Some(1) {
						*version_field = json!("1");
					} else if version.as_u64() == Some(2) {
						*version_field = json!("2");
					}
				}
			}
		}

		let versioned_library: VersionedLibrary =
			serde_json::from_value(value).context("Error parsing library file")?;
		Ok(versioned_library)
	}

	pub type Library = V2Library;

	#[derive(Deserialize, Clone, Debug)]
	#[serde(deny_unknown_fields)]
	pub struct V2Library {
		pub tracks: LinkedHashMap<TrackID, Track>,
		pub trackLists: TrackLists,
		/// v1 playtime has two issues:
		/// - some durations are double counted (or triple, etc.)
		/// - timestamps aren't updated after pausing
		pub v1PlayTime: Vec<PlayTime>,
		pub playTime: Vec<PlayTime>,
	}

	#[derive(Deserialize, Clone, Debug)]
	#[serde(tag = "version", deny_unknown_fields)]
	enum VersionedLibrary {
		#[serde(rename = "1")]
		V1(V1Library),
		#[serde(rename = "2")]
		V2(V2Library),
	}
	impl VersionedLibrary {
		pub fn upgrade(self) -> V2Library {
			match self {
				VersionedLibrary::V1(v1) => v1.upgrade(),
				VersionedLibrary::V2(v2) => v2,
			}
		}
	}

	#[derive(Deserialize, Clone, Debug)]
	#[serde(deny_unknown_fields)]
	struct V1Library {
		tracks: LinkedHashMap<TrackID, Track>,
		trackLists: TrackLists,
		playTime: Vec<PlayTime>,
	}
	impl V1Library {
		fn upgrade<'a>(self) -> V2Library {
			V2Library {
				tracks: self.tracks,
				trackLists: self.trackLists,
				v1PlayTime: self.playTime,
				playTime: Vec::new(),
			}
		}
	}

	type TrackID = String;
	type TrackListID = String;
	type MsSinceUnixEpoch = i64;
	/// Should be 0-100
	type PercentInteger = u8;
	pub type TrackLists = LinkedHashMap<TrackListID, TrackList>;

	/// (track id, start time, duration)
	type PlayTime = (TrackID, MsSinceUnixEpoch, i64);

	#[derive(Deserialize, Clone, Debug)]
	pub struct Track {
		pub size: i64,
		pub duration: f64,
		pub bitrate: f64,
		pub sampleRate: f64,
		pub file: String,
		pub dateModified: MsSinceUnixEpoch,
		pub dateAdded: MsSinceUnixEpoch,
		pub name: String,
		#[serde(default)]
		pub importedFrom: Option<String>,
		/// Imported ID, like iTunes Persistent ID
		#[serde(default)]
		pub originalId: Option<String>,
		#[serde(default)]
		pub artist: String,
		#[serde(default)]
		pub composer: Option<String>,
		#[serde(default)]
		pub sortName: Option<String>,
		#[serde(default)]
		pub sortArtist: Option<String>,
		#[serde(default)]
		pub sortComposer: Option<String>,
		#[serde(default)]
		pub genre: Option<String>,
		#[serde(default)]
		pub rating: Option<PercentInteger>,
		#[serde(default)]
		pub year: Option<i64>,
		#[serde(default)]
		pub bpm: Option<f64>,
		#[serde(default)]
		pub comments: Option<String>,
		#[serde(default)]
		pub grouping: Option<String>,
		#[serde(default)]
		pub liked: Option<bool>,
		#[serde(default)]
		pub disliked: Option<bool>,
		#[serde(default)]
		pub disabled: Option<bool>,
		#[serde(default)]
		pub compilation: Option<bool>,
		#[serde(default)]
		pub albumName: Option<String>,
		#[serde(default)]
		pub albumArtist: Option<String>,
		#[serde(default)]
		pub sortAlbumName: Option<String>,
		#[serde(default)]
		pub sortAlbumArtist: Option<String>,
		#[serde(default)]
		pub trackNum: Option<u32>,
		#[serde(default)]
		pub trackCount: Option<u32>,
		#[serde(default)]
		pub discNum: Option<u32>,
		#[serde(default)]
		pub discCount: Option<u32>,
		#[serde(default)]
		pub dateImported: Option<MsSinceUnixEpoch>,
		#[serde(default)]
		pub playCount: Option<u32>,
		#[serde(default)]
		pub plays: Option<Vec<MsSinceUnixEpoch>>,
		#[serde(default)]
		pub playsImported: Option<Vec<CountObject>>,
		#[serde(default)]
		pub skipCount: Option<u32>,
		#[serde(default)]
		pub skips: Option<Vec<MsSinceUnixEpoch>>,
		#[serde(default)]
		pub skipsImported: Option<Vec<CountObject>>,
		/// -100 to 100
		#[serde(default)]
		pub volume: Option<i8>,
	}

	#[derive(Deserialize, Clone, Debug)]
	pub struct CountObject {
		pub count: i64,
		pub fromDate: MsSinceUnixEpoch,
		pub toDate: MsSinceUnixEpoch,
	}

	#[derive(Deserialize, Clone, Debug)]
	#[serde(tag = "type")]
	pub enum TrackList {
		#[serde(rename = "playlist")]
		Playlist(Playlist),
		#[serde(rename = "folder")]
		Folder(Folder),
		#[serde(rename = "special")]
		Special(Special),
	}

	#[derive(Deserialize, Clone, Debug)]
	pub struct Playlist {
		pub id: TrackListID,
		pub name: String,
		#[serde(default)]
		pub description: Option<String>,
		#[serde(default)]
		pub liked: bool,
		#[serde(default)]
		pub disliked: bool,
		#[serde(default)]
		pub importedFrom: Option<String>,
		#[serde(default)]
		pub originalId: Option<String>,
		#[serde(default)]
		pub dateImported: Option<MsSinceUnixEpoch>,
		#[serde(default)]
		pub dateCreated: Option<MsSinceUnixEpoch>,
		pub tracks: Vec<TrackID>,
	}

	#[derive(Deserialize, Clone, Debug)]
	pub struct Folder {
		pub id: TrackListID,
		pub name: String,
		#[serde(default)]
		pub description: Option<String>,
		#[serde(default)]
		pub liked: bool,
		#[serde(default)]
		pub disliked: bool,
		/// For example "itunes"
		#[serde(default)]
		pub importedFrom: Option<String>,
		/// For example iTunes Persistent ID
		#[serde(default)]
		pub originalId: Option<String>,
		#[serde(default)]
		pub dateImported: Option<MsSinceUnixEpoch>,
		#[serde(default)]
		pub dateCreated: Option<MsSinceUnixEpoch>,
		pub children: Vec<TrackListID>,
	}

	#[derive(Deserialize, Clone, Debug)]
	pub struct Special {
		pub id: TrackListID,
		pub name: SpecialTrackListName,
		pub dateCreated: MsSinceUnixEpoch,
		pub children: Vec<TrackListID>,
	}

	#[derive(Deserialize, Clone, Debug)]
	pub enum SpecialTrackListName {
		Root,
	}
	impl ToString for SpecialTrackListName {
		fn to_string(&self) -> String {
			match self {
				SpecialTrackListName::Root => "Root".to_owned(),
			}
		}
	}
}
