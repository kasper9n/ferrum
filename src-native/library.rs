#[cfg(feature = "napi-rs")]
use crate::data::Data;
use crate::library_types::{ItemId, Library, SpecialTrackListName, TrackList, VersionedLibrary};
#[cfg(feature = "napi-rs")]
use crate::migrate::migrate_to_sqlite;
use anyhow::{Context, Result, bail};
use linked_hash_map::LinkedHashMap;
use serde_json::{Value, json};
#[cfg(feature = "napi-rs")]
use sqlx::{ConnectOptions, SqliteConnection, sqlite::SqliteConnectOptions};
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::PathBuf;
#[cfg(feature = "napi-rs")]
use std::time::Instant;
#[cfg(feature = "napi-rs")]
use std::{fs::create_dir_all, path::Path};
#[cfg(feature = "napi-rs")]
use tokio::runtime::Runtime;

#[cfg(feature = "napi-rs")]
#[derive(Clone)]
#[napi(object)]
pub struct Paths {
	pub path_separator: String,
	pub library_dir: String,
	pub tracks_dir: String,
	pub library_sqlite: String,
	pub library_json: String,
	pub cache_dir: String,
	pub cache_db: String,
	pub local_data_dir: String,
	pub view_options_file: String,
	pub queue_file: String,
	pub logs_dir: String,
}
#[cfg(feature = "napi-rs")]
impl Paths {
	fn ensure_dirs_exists(&self) -> Result<()> {
		create_dir_all(&self.library_dir)?;
		create_dir_all(&self.tracks_dir)?;
		create_dir_all(&self.cache_dir)?;
		create_dir_all(&self.local_data_dir)?;
		// We do not create logs_dir, we create it lazily when a crash occurs
		return Ok(());
	}
	pub fn get_track_file_path(&self, file: &str) -> PathBuf {
		PathBuf::from(&self.tracks_dir).join(file)
	}
}

// todo: delete
pub fn load_library_json(library_json: &PathBuf) -> Result<Option<Library>> {
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

	let library = versioned_library.upgrade().init_libary();
	Ok(Some(library))
}

// todo: delete
fn parse_old_versionless_library_json(library_file: &mut File) -> Result<VersionedLibrary<'_>> {
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

#[cfg(feature = "napi-rs")]
pub fn open_library(paths: &Paths) -> Result<SqliteConnection> {
	let now = Instant::now();

	paths
		.ensure_dirs_exists()
		.context("Error ensuring folder exists")?;
	println!("Loading library at path: {}", paths.library_dir);

	let library_sqlite = &paths.library_sqlite;

	let rt = Runtime::new().context("Error creating tokio runtime")?;

	let exists = Path::new(&library_sqlite).exists();
	if !exists {
		rt.block_on(migrate_to_sqlite(paths))?;
	}
	let mut connection = rt
		.block_on(
			SqliteConnectOptions::new()
				.filename(&paths.library_sqlite)
				.connect(),
		)
		.context("Error connecting to library database")?;

	rt.block_on(sqlx::migrate!("./src-native/migrations").run(&mut connection))
		.map_err(|e| anyhow::anyhow!("{:?}", e))
		.context("Could not run database migrations")?;

	println!("Open library: {}ms", now.elapsed().as_millis());
	Ok(connection)
}

pub enum TrackField {
	String,
	F64,
	I64,
	U32,
	I8,
	U8,
	Bool,
}

#[cfg(feature = "napi-rs")]
#[napi(js_name = "get_default_sort_desc")]
#[allow(dead_code)]
pub fn get_default_sort_desc(field: String) -> Result<bool> {
	if field == "index" {
		return Ok(true);
	}
	let field = get_track_field_type(&field)?;
	let desc = match field {
		TrackField::String => false,
		_ => true,
	};
	Ok(desc)
}

pub fn get_track_field_type(field: &str) -> Result<TrackField> {
	let field = match field {
		"size" => TrackField::I64,
		"duration" => TrackField::F64,
		"bitrate" => TrackField::F64,
		"sampleRate" => TrackField::F64,
		"file" => TrackField::String,
		"dateModified" => TrackField::I64,
		"dateAdded" => TrackField::I64,
		"name" => TrackField::String,
		"importedFrom" => TrackField::String,
		"originalId" => TrackField::String,
		"artist" => TrackField::String,
		"composer" => TrackField::String,
		"sortName" => TrackField::String,
		"sortArtist" => TrackField::String,
		"sortComposer" => TrackField::String,
		"genre" => TrackField::String,
		"rating" => TrackField::U8,
		"year" => TrackField::I64,
		"bpm" => TrackField::F64,
		"comments" => TrackField::String,
		"grouping" => TrackField::String,
		"liked" => TrackField::Bool,
		"disliked" => TrackField::Bool,
		"disabled" => TrackField::Bool,
		"compilation" => TrackField::Bool,
		"albumName" => TrackField::String,
		"albumArtist" => TrackField::String,
		"sortAlbumName" => TrackField::String,
		"sortAlbumArtist" => TrackField::String,
		"trackNum" => TrackField::U32,
		"trackCount" => TrackField::U32,
		"discNum" => TrackField::U32,
		"discCount" => TrackField::U32,
		"dateImported" => TrackField::I64,
		"playCount" => TrackField::U32,
		"skipCount" => TrackField::U32,
		"volume" => TrackField::I8,
		_ => bail!("Field type not found for {}", field),
	};
	return Ok(field);
}

#[cfg(feature = "napi-rs")]
#[napi(js_name = "get_genres")]
#[allow(dead_code)]
pub fn get_genres() -> Vec<String> {
	let mut data = Data::get_blocking();
	let genres = data.library.get_genres();
	genres.clone()
}

#[cfg(feature = "napi-rs")]
#[napi(js_name = "get_artists")]
#[allow(dead_code)]
pub fn get_artists() -> Vec<String> {
	let mut data = Data::get_blocking();
	let genres = data.library.get_artists();
	genres.clone()
}

pub fn get_tracklist_item_ids(library: &Library, playlist_id: &str) -> Result<Vec<ItemId>> {
	match library.get_tracklist(playlist_id)? {
		TrackList::Playlist(playlist) => Ok(playlist.tracks.clone()),
		TrackList::Folder(folder) => {
			let mut ids: LinkedHashMap<ItemId, ()> = LinkedHashMap::new();
			for child in &folder.children {
				let child_ids = get_tracklist_item_ids(library, &child)?;
				for child_id in child_ids {
					ids.insert(child_id, ());
				}
			}
			Ok(ids.into_iter().map(|(id, _)| id).collect())
		}
		TrackList::Special(special) => match special.name {
			SpecialTrackListName::Root => {
				let item_ids = library.get_track_item_ids().values().cloned().collect();
				Ok(item_ids)
			}
		},
	}
}
