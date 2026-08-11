use crate::data::Data;
use crate::get_now_timestamp;
use crate::library::Paths;
use crate::library_types::{
	CountObject, Folder, Library, Playlist, Track, TrackList, new_item_ids_from_track_ids,
};
use crate::tracks::generate_filename;
use crate::tracks::import::{FileType, read_file_metadata};
use anyhow::{Context, Result, bail};
use lofty::file::{AudioFile, TaggedFileExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use time::OffsetDateTime;
use time::serde::iso8601;
use time::serde::iso8601::option as iso8601_opt;

#[derive(Serialize, Deserialize, Debug)]
struct XmlLibrary {
	#[serde(rename = "Major Version")]
	major_version: u16,

	#[serde(rename = "Minor Version")]
	minor_version: u16,

	#[serde(rename = "Date")]
	date: String, // 2022-10-14T05:12:37Z

	#[serde(rename = "Application Version")]
	application_version: String,

	#[serde(rename = "Features")]
	features: u16,

	#[serde(rename = "Show Content Ratings")]
	show_content_ratings: bool,

	#[serde(rename = "Music Folder")]
	music_folder: String,

	#[serde(rename = "Library Persistent ID")]
	library_persistent_id: String,

	#[serde(rename = "Tracks")]
	tracks: HashMap<String, plist::Value>,

	#[serde(rename = "Playlists")]
	playlists: Vec<plist::Value>,
}
impl XmlLibrary {
	fn deserialize_props(self) -> Result<XmlLibraryProps> {
		let mut tracks = HashMap::new();
		for (key, value) in self.tracks {
			let podcast = value
				.as_dictionary()
				.and_then(|d| d.get("Podcast"))
				.and_then(|v| v.as_boolean())
				.unwrap_or(false);
			if podcast {
				continue;
			}

			let track: XmlTrack = plist::from_value(&value)
				.with_context(|| format!("Could not read track with id \"{key}"))?;
			tracks.insert(key, track);
		}
		let mut playlists = Vec::new();
		for value in &self.playlists {
			let name = value
				.as_dictionary()
				.and_then(|dict| dict.get("Name"))
				.and_then(|v| v.as_string())
				.unwrap_or_default()
				.to_string();
			let playlist: XmlPlaylist = plist::from_value(value)
				.with_context(|| format!("Could not read playlist \"{name}\""))?;
			playlists.push(playlist);
		}
		Ok(XmlLibraryProps { tracks, playlists })
	}
}

struct XmlLibraryProps {
	tracks: HashMap<String, XmlTrack>,
	playlists: Vec<XmlPlaylist>,
}

impl XmlLibraryProps {
	fn get_music_playlist(playlists: &Vec<XmlPlaylist>) -> Result<&XmlPlaylist> {
		let mut xml_music_playlist = None;
		for xml_playlist in playlists {
			if xml_playlist.distinguished_kind == Some(4) {
				if xml_music_playlist.is_some() {
					bail!("Found two iTunes-generated Music playlists");
				} else {
					xml_music_playlist = Some(xml_playlist);
				}
			}
		}
		xml_music_playlist.context("No Music playlist found")
	}
	fn take_importable_playlists(&mut self) -> Vec<XmlPlaylist> {
		let playlists = std::mem::take(&mut self.playlists);
		let (importable, remaining) = playlists.into_iter().partition(|xml_playlist| {
			return xml_playlist.is_importable_playlist();
		});
		self.playlists = remaining;
		importable
	}
}

fn get_none<T>() -> Option<T> {
	None
}

#[derive(Serialize, Deserialize, Debug)]
struct XmlTrack {
	// Artist
	#[serde(rename = "Artist")]
	artist: Option<String>,

	// Name
	#[serde(rename = "Name")]
	name: Option<String>,

	// Composer
	#[serde(rename = "Composer")]
	composer: Option<String>,

	// Sort Name
	#[serde(rename = "Sort Name")]
	sort_name: Option<String>,

	// Sort Artist
	#[serde(rename = "Sort Artist")]
	sort_artist: Option<String>,

	// Sort Composer
	#[serde(rename = "Sort Composer")]
	sort_composer: Option<String>,

	// Genre
	#[serde(rename = "Genre")]
	genre: Option<String>,

	// Size
	#[serde(rename = "Size")]
	size: Option<u64>,

	// Rating
	#[serde(rename = "Rating")]
	rating: Option<u8>,

	// Year
	#[serde(rename = "Year")]
	year: Option<i64>,

	// BPM
	#[serde(rename = "BPM")]
	bpm: Option<u32>,

	// Date Modified
	#[serde(with = "iso8601", rename = "Date Modified")]
	date_modified: OffsetDateTime,

	// Date Added
	#[serde(with = "iso8601", rename = "Date Added")]
	date_added: OffsetDateTime,

	// Comments
	#[serde(rename = "Comments")]
	comments: Option<String>,

	// Grouping
	#[serde(rename = "Grouping")]
	grouping: Option<String>,

	// Play Date
	#[serde(rename = "Play Date")]
	play_date: Option<u64>,

	// Play Date UTC
	#[serde(with = "iso8601_opt", default = "get_none", rename = "Play Date UTC")]
	play_date_utc: Option<OffsetDateTime>,

	// Play Count
	#[serde(rename = "Play Count")]
	play_count: Option<u32>,

	// Skip Date
	#[serde(with = "iso8601_opt", default = "get_none", rename = "Skip Date")]
	skip_date: Option<OffsetDateTime>,

	// Skip Count
	#[serde(rename = "Skip Count")]
	skip_count: Option<u32>,

	// Volume Adjustment (-255 to 255)
	#[serde(rename = "Volume Adjustment")]
	volume_adjustment: Option<i16>,

	// Loved
	#[serde(rename = "Loved")]
	loved: Option<bool>,

	// Disliked
	#[serde(rename = "Disliked")]
	disliked: Option<bool>,

	// Disabled
	#[serde(rename = "Disabled")]
	disabled: Option<bool>,

	// Compilation
	#[serde(rename = "Compilation")]
	compilation: Option<bool>,

	// Album
	#[serde(rename = "Album")]
	album: Option<String>,

	// Album Artist
	#[serde(rename = "Album Artist")]
	album_artist: Option<String>,

	// Sort Album
	#[serde(rename = "Sort Album")]
	sort_album: Option<String>,

	// Sort Album Artist
	#[serde(rename = "Sort Album Artist")]
	sort_album_artist: Option<String>,

	// Track Number
	#[serde(rename = "Track Number")]
	track_number: Option<u32>,

	// Track Count
	#[serde(rename = "Track Count")]
	track_count: Option<u32>,

	// Disc Number
	#[serde(rename = "Disc Number")]
	disc_number: Option<u32>,

	// Disc Count
	#[serde(rename = "Disc Count")]
	disc_count: Option<u32>,

	// Persistent ID
	#[serde(rename = "Persistent ID")]
	persistent_id: String,

	// Track Type
	#[serde(rename = "Track Type")]
	track_type: Option<String>,

	// Location
	#[serde(rename = "Location")]
	location: Option<String>,
}
impl XmlTrack {
	fn artist_title(&self) -> String {
		self.artist.as_deref().unwrap_or_default().to_string()
			+ self.name.as_deref().unwrap_or_default()
	}
}

fn datetime_to_timestamp_millis(datetime: OffsetDateTime) -> i64 {
	datetime.unix_timestamp() * 1000 + datetime.millisecond() as i64
}
fn keep_true(value: Option<bool>) -> Option<bool> {
	value.filter(|v| *v)
}
fn keep_filled(value: Option<String>) -> Option<String> {
	value.filter(|v| v != "")
}

struct CountInfo {
	count: Option<u32>,
	imported: Option<u32>,
	dates: Option<Vec<i64>>,
}
impl CountInfo {
	fn new(count: Option<u32>, date: Option<OffsetDateTime>) -> CountInfo {
		let mut imported_count = count.unwrap_or(0);
		if imported_count == 0 {
			return CountInfo {
				count,
				imported: None,
				dates: None,
			};
		}
		// If we have a date (like play_date), add a play for it
		let dates = date.map(|date| {
			imported_count -= 1;
			vec![datetime_to_timestamp_millis(date)]
		});
		CountInfo {
			count,
			imported: Some(imported_count),
			dates,
		}
	}
}

fn parse_file_url(value: &str) -> Result<PathBuf> {
	let file_url = url::Url::parse(value).context("Invalid track location")?;
	match file_url.scheme() {
		"file" => {}
		_ => bail!("Invalid track location scheme: {}", value),
	}
	match file_url.to_file_path() {
		Ok(path) => Ok(path),
		Err(()) => bail!("Invalid track location host: {}", value),
	}
}

/// Parses track but does not move it to `tracks_dir`
fn parse_track(xml_track: XmlTrack, start_time: i64, paths: &Paths) -> Result<(PathBuf, Track)> {
	let xml_location = xml_track.location.context("Missing track location")?;
	if xml_track.track_type != Some("File".to_string()) {
		bail!(
			"Track with type {}, expected \"File\"",
			xml_track.track_type.as_deref().unwrap_or("unknown"),
		);
	}

	// Unlike "Skip Date" etc, "Play Date" is a non-UTC Mac HFS+ timestamp, but
	// luckily "Play Date UTC" is a normal date.
	let play = CountInfo::new(xml_track.play_count, xml_track.play_date_utc);

	let skip = CountInfo::new(xml_track.skip_count, xml_track.skip_date);

	let xml_track_path = parse_file_url(&xml_location)?;

	// this will also checks if the file exists
	let file_md = read_file_metadata(&xml_track_path)?;

	let tagged_file =
		lofty::read_from_path(&xml_track_path).context("Failed to read file information")?;
	let audio_properties = tagged_file.properties();

	let file_type = FileType::from_path(&xml_track_path)?;
	{
		// check that file type matches file extension
		let lofty_file_type = FileType::from_lofty_file_type(tagged_file.file_type())?;
		if file_type != lofty_file_type {
			println!("File type mismatch: {} vs {}", file_type, lofty_file_type);
		}
	}

	let name = xml_track.name.unwrap_or_default();
	let artist = xml_track.artist.unwrap_or_default();
	let filename = generate_filename(&paths, &artist, &name, file_type.file_extension());

	let track = Track {
		size: file_md.len() as i64,
		duration: audio_properties.duration().as_secs_f64(),
		bitrate: audio_properties
			.audio_bitrate()
			.context("Unknown bitrate")?
			.into(),
		sampleRate: audio_properties
			.audio_bitrate()
			.context("Unknown sample rate")?
			.into(),
		file: filename,
		dateModified: datetime_to_timestamp_millis(xml_track.date_modified),
		dateAdded: datetime_to_timestamp_millis(xml_track.date_added),
		name,
		importedFrom: Some("itunes".to_string()),
		originalId: Some(xml_track.persistent_id),
		artist,
		composer: keep_filled(xml_track.composer),
		sortName: keep_filled(xml_track.sort_name),
		sortArtist: keep_filled(xml_track.sort_artist),
		sortComposer: keep_filled(xml_track.sort_composer),
		genre: keep_filled(xml_track.genre),
		rating: xml_track.rating,
		year: xml_track.year,
		bpm: xml_track.bpm.map(|bpm| bpm.into()),
		comments: keep_filled(xml_track.comments),
		grouping: keep_filled(xml_track.grouping),
		liked: keep_true(xml_track.loved),
		disliked: keep_true(xml_track.disliked),
		disabled: keep_true(xml_track.disabled),
		compilation: keep_true(xml_track.compilation),
		albumName: keep_filled(xml_track.album),
		albumArtist: keep_filled(xml_track.album_artist),
		sortAlbumName: keep_filled(xml_track.sort_album),
		sortAlbumArtist: keep_filled(xml_track.sort_album_artist),
		trackNum: xml_track.track_number,
		trackCount: xml_track.track_count,
		discNum: xml_track.disc_number,
		discCount: xml_track.disc_count,
		dateImported: Some(start_time),
		playCount: play.count,
		plays: play.dates,
		playsImported: play.imported.map(|count| {
			vec![CountObject {
				count: count.into(),
				fromDate: datetime_to_timestamp_millis(xml_track.date_added),
				toDate: start_time,
			}]
		}),
		skipCount: skip.count,
		skips: skip.dates,
		skipsImported: skip.imported.map(|count| {
			vec![CountObject {
				count: count.into(),
				fromDate: datetime_to_timestamp_millis(xml_track.date_added),
				toDate: start_time,
			}]
		}),
		volume: match xml_track.volume_adjustment {
			Some(0) | None => None,
			Some(volume_adjustment) => {
				let float: f32 = volume_adjustment.into();
				let vol = (float / 2.55).round() as i8;
				if vol < -100 || vol > 100 {
					bail!("Invalid volume adjustment: {}", volume_adjustment);
				}
				Some(vol)
			}
		},
	};

	Ok((xml_track_path, track))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XmlPlaylist {
	#[serde(rename = "Name")]
	name: String,

	#[serde(rename = "Description")]
	description: Option<String>,

	#[serde(rename = "Loved")]
	loved: Option<bool>,

	#[serde(rename = "Disliked")]
	disliked: Option<bool>,

	#[serde(rename = "Visible")]
	visible: Option<bool>,

	#[serde(rename = "Smart Info")]
	smart_info: Option<plist::Value>,

	#[serde(rename = "Playlist Persistent ID")]
	playlist_persistent_id: String,

	#[serde(rename = "Parent Persistent ID")]
	parent_persistent_id: Option<String>,

	/// Matches the type from the docs, but is 1-indexed instead of 0-indexed
	/// https://developer.apple.com/documentation/ituneslibrary/itlibdistinguishedplaylistkind
	#[serde(rename = "Distinguished Kind")]
	distinguished_kind: Option<u64>,

	#[serde(rename = "Folder")]
	folder: Option<bool>,

	#[serde(rename = "Playlist Items")]
	playlist_items: Option<Vec<XmlPlaylistItem>>,
}
impl XmlPlaylist {
	fn is_importable_playlist(&self) -> bool {
		// invisible playlists (should just be the "Library" playlist)
		if self.visible == Some(false) {
			return false;
		};
		// smart playlists
		if self.smart_info.is_some() {
			return false;
		};
		match self.distinguished_kind {
			None | Some(1) => return true,
			Some(_) => return false, // ignore special iTunes playlists
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XmlPlaylistItem {
	#[serde(rename = "Track ID")]
	track_id: u64,
}

struct XmlPlaylistInfo {
	xml_playlist: XmlPlaylist,
	child_indexes: Vec<usize>,
}

/// Returns id of the imported playlist
fn import_playlist(
	infos: &Vec<XmlPlaylistInfo>,
	i: usize,
	library: &mut Library,
	start_time: i64,
	errors: &mut Vec<String>,
	xml_track_id_map: &HashMap<String, String>,
) -> String {
	let tracklist;
	let xml_playlist = &infos[i].xml_playlist;
	let id = library.generate_id();

	if xml_playlist.folder == Some(true) {
		let folder = TrackList::Folder(Folder {
			id: id.clone(),
			name: xml_playlist.name.clone(),
			description: xml_playlist.description.clone(),
			liked: xml_playlist.loved.unwrap_or_default(),
			disliked: xml_playlist.disliked.unwrap_or_default(),
			importedFrom: Some("itunes".to_string()),
			originalId: Some(xml_playlist.playlist_persistent_id.clone()),
			dateImported: Some(start_time),
			dateCreated: None,
			children: infos[i]
				.child_indexes
				.iter()
				.map(|child_i| {
					return import_playlist(
						&infos,
						*child_i,
						library,
						start_time,
						errors,
						xml_track_id_map,
					);
				})
				.collect(),
		});
		// immediately insert into library so new generated ids are unique
		library.trackLists.insert(id.clone(), folder);
	} else {
		let mut track_ids = Vec::new();
		for playlist_item in xml_playlist.playlist_items.as_ref().unwrap() {
			let track_id = xml_track_id_map.get(&playlist_item.track_id.to_string());
			match track_id {
				Some(track_id) => track_ids.push(track_id.clone()),
				None => errors.push(format!(
					"Track with id {} not found in playlist {}",
					playlist_item.track_id, xml_playlist.name
				)),
			}
		}

		tracklist = TrackList::Playlist(Playlist {
			id: id.clone(),
			name: xml_playlist.name.clone(),
			description: xml_playlist.description.clone(),
			liked: xml_playlist.loved.unwrap_or_default(),
			disliked: xml_playlist.disliked.unwrap_or_default(),
			importedFrom: Some("itunes".to_string()),
			originalId: Some(xml_playlist.playlist_persistent_id.clone()),
			dateImported: Some(start_time),
			dateCreated: None,
			tracks: new_item_ids_from_track_ids(&track_ids),
		});
		// immediately insert into library so new generated ids are unique
		library.trackLists.insert(id.clone(), tracklist);
	}
	id
}

#[napi(object)]
pub struct ImportStatus {
	pub errors: Vec<String>,
	pub tracks_count: i64,
	pub playlists_count: i64,
}

#[napi]
pub struct ItunesImport {
	new_library: Mutex<Option<Library>>,
	/// iTunes path -> Ferrum file
	itunes_track_paths: Mutex<HashMap<PathBuf, String>>,
	paths: Paths,
}
#[napi]
impl ItunesImport {
	#[napi(factory)]
	pub fn new() -> Self {
		let data = Data::get_blocking();
		Self {
			new_library: Some(data.library.clone()).into(),
			itunes_track_paths: HashMap::new().into(),
			paths: data.paths.clone(),
		}
	}
	#[napi]
	pub async fn start(&self, path: String) -> napi::Result<ImportStatus> {
		Ok(import_itunes(self, path).await?)
	}
	#[napi]
	pub fn finish(&mut self) -> napi::Result<()> {
		let mut data = Data::get_blocking();
		let itunes_track_paths = &mut *self.itunes_track_paths.lock().unwrap();
		for (itunes_path, ferrum_file) in itunes_track_paths {
			let new_path = data.paths.get_track_file_path(ferrum_file);
			fs::copy(itunes_path, new_path).context("Error copying file")?;
		}
		let new_library = &mut self.new_library.lock().unwrap();
		data.library = new_library.take().context("Not initialized")?;
		Ok(())
	}
}

async fn import_itunes(itunes_import: &ItunesImport, path: String) -> Result<ImportStatus> {
	let new_library_lock = &mut *itunes_import.new_library.lock().unwrap();
	let mut library = match new_library_lock {
		Some(library) => library,
		None => bail!("Not initialized"),
	};
	let mut itunes_track_paths = itunes_import.itunes_track_paths.lock().unwrap();
	let original_tracks_count = library.get_tracks().len();
	let original_tracklists_count = library.trackLists.len();
	let xml_lib: XmlLibrary = plist::from_file(path).context("Unable to parse")?;
	let mut errors = Vec::new();
	let start_time = get_now_timestamp();

	// Library.xml version check
	let version =
		xml_lib.major_version.to_string() + "." + xml_lib.minor_version.to_string().as_str();
	if (xml_lib.major_version, xml_lib.minor_version) != (1, 1) {
		errors.push(format!("Unsupported Library.xml version {version}"));
	}

	let mut xml = xml_lib.deserialize_props()?;

	// iTunes ID -> Ferrum ID
	let mut xml_track_id_map = HashMap::<String, String>::new();

	// We import the tracks that are in the "Music" playlist since xml.tracks
	// contains podcasts, etc.
	let xml_music_playlist = XmlLibraryProps::get_music_playlist(&xml.playlists)?;
	let playlist_items = &xml_music_playlist.playlist_items.as_ref().unwrap();
	let track_count = playlist_items.len();
	for (i, playlist_item) in playlist_items.iter().enumerate() {
		let xml_id = playlist_item.track_id.to_string();
		println!("Parsing tracks {}/{}", i + 1, track_count);
		let xml_track = xml
			.tracks
			.remove(&xml_id)
			.with_context(|| format!("Track with id {} not found", playlist_item.track_id))?;
		let artist_title = xml_track.artist_title();

		if matches!(xml_track.name.as_deref(), Some("") | None) {
			errors.push(format!("Missing track name: {artist_title}"));
		}
		if matches!(xml_track.artist.as_deref(), Some("") | None) {
			errors.push(format!("Missing track artist: {artist_title}"));
		}

		match parse_track(xml_track, start_time, &itunes_import.paths) {
			Ok((xml_track_path, track)) => {
				let generated_id = library.generate_id();
				// immediately insert into library so new generated ids are unique
				itunes_track_paths.insert(xml_track_path, track.file.clone());
				library.insert_track(generated_id.clone(), track);
				if xml_track_id_map.contains_key(&xml_id) {
					errors.push(format!("Duplicate track ids \"{}\": artist_title", xml_id));
				}
				xml_track_id_map.insert(xml_id, generated_id);
			}
			Err(e) => {
				errors.push(format!("[{artist_title}] Skipped track: {e}"));
			}
		};
	}

	let importable_xml_playlists = {
		let mut list = xml.take_importable_playlists();
		for xml_playlist in &mut list {
			if xml_playlist.playlist_items.is_none() {
				errors.push(format!(
					"No playlist items list in playlist {}",
					xml_playlist.name
				));
				xml_playlist.playlist_items = Some(Vec::new());
			}
		}
		list
	};

	let xml_playlist_id_map = {
		let mut map = HashMap::<String, usize>::new();
		for (i, playlist) in importable_xml_playlists.iter().enumerate() {
			map.insert(playlist.playlist_persistent_id.clone(), i);
		}
		map
	};

	let (xml_playlist_infos, root_child_indexes) = {
		// create infos
		let mut xml_playlist_infos: Vec<_> = importable_xml_playlists
			.iter()
			.map(|p| {
				return XmlPlaylistInfo {
					xml_playlist: p.clone(),
					child_indexes: Vec::new(),
				};
			})
			.collect();

		let mut root_child_indexes = Vec::new();

		// add children to infos
		for (i, xml_playlist) in importable_xml_playlists.iter().enumerate() {
			if let Some(parent_id) = &xml_playlist.parent_persistent_id {
				let parent_index = match xml_playlist_id_map.get(parent_id) {
					Some(index) => index,
					None => {
						errors.push(format!(
							"Playlist \"{}\" has non-existent parent id {}",
							xml_playlist.name, parent_id
						));
						continue;
					}
				};
				let parent = &mut xml_playlist_infos[*parent_index];
				parent.child_indexes.push(i);
			} else {
				root_child_indexes.push(i);
			}
		}

		(xml_playlist_infos, root_child_indexes)
	};

	// recursively import playlists
	for i in root_child_indexes {
		let playlist_id = import_playlist(
			&xml_playlist_infos,
			i,
			&mut library,
			start_time,
			&mut errors,
			&xml_track_id_map,
		);
		let root = library.get_root_tracklist_mut()?;
		root.children.push(playlist_id);
	}

	Ok(ImportStatus {
		errors,
		tracks_count: (library.get_tracks().len() - original_tracks_count) as i64,
		playlists_count: (library.trackLists.len() - original_tracklists_count) as i64,
	})
}

#[tokio::test]
async fn import_itunes_test() {
	let library_xml_path = "src-native/tests/Library.xml".to_string();
	let _xml: XmlLibrary = plist::from_file(library_xml_path).unwrap();
}
