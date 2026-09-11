#[cfg(feature = "napi-rs")]
use crate::data::Data;
use crate::{
	library_types::{ItemId, Library, SpecialTrackListName, TrackList, TrackListID},
	migrate::{self, LibraryFile, parse_old_version_library_json},
};
use anyhow::{Context, Result, bail};
use linked_hash_map::LinkedHashMap;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone)]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Paths {
	pub path_separator: String,
	pub library_dir: String,
	pub tracks_dir: String,
	pub library_json: String,
	pub cache_dir: String,
	pub cache_db: String,
	pub local_data_dir: String,
	pub view_options_file: String,
	pub queue_file: String,
	pub logs_dir: String,
}
impl Paths {
	pub fn ensure_dirs_exists(&self) -> Result<()> {
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

#[cfg(feature = "napi-rs")]
pub fn load_library(paths: &Paths) -> Result<Library> {
	paths
		.ensure_dirs_exists()
		.context("Error ensuring folder exists")?;
	println!("Loading library at path: {}", paths.library_dir);

	load_library_from_file(&paths)
}

pub fn load_library_from_file(paths: &Paths) -> Result<Library> {
	let t = Instant::now();

	let mut library_file = match File::open(&paths.library_json) {
		Ok(file) => file,
		Err(err) => match err.kind() {
			ErrorKind::NotFound => return Ok(Library::new()),
			_ => return Err(err).context("Error opening library file"),
		},
	};

	let mut json_bytes = Vec::new();
	library_file
		.read_to_end(&mut json_bytes)
		.context("Error reading library file")?;

	println!("Read library: {}ms", t.elapsed().as_millis());
	let t = Instant::now();

	let versioned_library: LibraryFile = match simd_json::from_slice(&mut json_bytes) {
		Ok(lib) => {
			println!("Parsed library: {}ms", t.elapsed().as_millis());
			lib
		}
		Err(_) => {
			library_file
				.seek(SeekFrom::Start(0))
				.context("Error seeking to start of library file")?;
			let versioned_library = parse_old_version_library_json(&mut library_file)?;
			println!("Parsed v0 library: {}ms", t.elapsed().as_millis());
			versioned_library
		}
	};
	let t = Instant::now();

	let latest_library = migrate::upgrade(versioned_library, paths)?;
	let library = Library::init_library(latest_library);
	println!("Initialized library: {}ms", t.elapsed().as_millis());
	Ok(library)
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

pub fn get_tracklist_item_ids(library: &Library, playlist_id: &TrackListID) -> Result<Vec<ItemId>> {
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
