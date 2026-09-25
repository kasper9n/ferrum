#![allow(non_snake_case)]

use crate::library::Paths;
pub(self) use crate::library_types as latest;
use crate::library_types::LatestLibrary;
use crate::{delete_file, save_overwrite};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use simd_json::base::{ValueAsMutObject, ValueAsScalar};
use simd_json::{OwnedValue, json};
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "version", deny_unknown_fields)]
pub enum LibraryFile<'a> {
	#[serde(rename = "1")]
	V1(v1::Library),
	#[serde(rename = "2")]
	V2(v2::Library),
	#[serde(rename = "3")]
	V3(LatestLibrary<'a>),
}
impl<'a> LibraryFile<'a> {
	pub fn latest_unwrap(self) -> LatestLibrary<'a> {
		match self {
			LibraryFile::V3(latest) => latest,
			_ => panic!("Expected latest library"),
		}
	}
}

/// For serialization, since we don't need to serialize old formats
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "version", deny_unknown_fields)]
pub enum LatestLibraryFile<'a> {
	#[serde(rename = "3")]
	V3(LatestLibrary<'a>),
}
impl LatestLibraryFile<'_> {
	pub fn save(&self, paths: &Paths) -> Result<()> {
		let now = Instant::now();
		let bytes = simd_json::to_vec(&self)?;
		println!("Stringify: {}ms", now.elapsed().as_millis());
		save_overwrite(bytes, &paths.library_json)?;
		Ok(())
	}
}

pub fn upgrade<'a>(versioned_library: LibraryFile<'a>, paths: &Paths) -> Result<LatestLibrary<'a>> {
	let already_latest = match versioned_library {
		LibraryFile::V3(LatestLibrary { .. }) => true,
		_ => false,
	};
	if !already_latest {
		let backup_path = PathBuf::from(&paths.library_backup_json);
		if backup_path.exists() {
			delete_file(&backup_path)?;
		}
		fs::copy(&paths.library_json, &paths.library_backup_json)
			.context("Failed to back up library (copy)")?;
		File::open(&paths.library_backup_json)
			.context("Failed to back up library (open)")?
			.sync_all()
			.context("Failed to back up library (sync)")?;
	}
	let latest = match versioned_library {
		LibraryFile::V1(v1) => v1.upgrade().upgrade(paths)?,
		LibraryFile::V2(v2) => v2.upgrade(paths)?,
		LibraryFile::V3(v3) => v3,
	};
	Ok(latest)
}

pub fn parse_old_version_library_json(library_file: &mut File) -> Result<LibraryFile<'_>> {
	let mut json_str = String::new();
	library_file
		.read_to_string(&mut json_str)
		.context("Error reading library file")?;
	let mut json_bytes = json_str.into_bytes();

	let mut value: OwnedValue =
		simd_json::deserialize(&mut json_bytes).context("Error parsing library file")?;
	// Migrate version number to string
	if let Some(obj) = value.as_object_mut() {
		if let Some(version_field) = obj.get_mut("version") {
			if let Some(version) = version_field.as_u64() {
				if version == 1 {
					*version_field = json!("1");
				} else if version == 2 {
					*version_field = json!("2");
				}
			}
		}
	}

	let versioned_library: LibraryFile =
		simd_json::serde::from_owned_value(value).context("Error parsing library file")?;
	Ok(versioned_library)
}

mod v1 {
	use crate::migrate::{latest, v2};
	use linked_hash_map::LinkedHashMap;
	use serde::Deserialize;

	#[derive(Deserialize, Clone, Debug)]
	#[serde(deny_unknown_fields)]
	pub struct Library {
		tracks: LinkedHashMap<TrackID, Track>,
		trackLists: TrackLists,
		playTime: Vec<PlayTime>,
	}
	impl Library {
		pub fn upgrade(self) -> v2::Library {
			v2::Library {
				tracks: self.tracks,
				trackLists: self.trackLists,
				// v1 playtime has two issues:
				// - some durations are double counted (or triple, etc.)
				// - timestamps aren't updated after pausing
				v1PlayTime: self.playTime,
				playTime: Vec::new(),
			}
		}
	}

	pub type TrackID = String;
	pub type TrackListID = String;
	pub type MsSinceUnixEpoch = i64;
	/// Should be 0-100
	// pub type PercentInteger = u8;
	pub type TrackLists = LinkedHashMap<TrackListID, TrackList>;

	/// (track id, start time, duration)
	pub type PlayTime = (TrackID, MsSinceUnixEpoch, i64);

	#[derive(Deserialize, Clone, Debug)]
	pub struct Track {
		pub size: i64,
		pub duration: f64,
		pub bitrate: f64,
		pub sampleRate: f64,
		pub file: String,
		pub dateModified: latest::MsSinceUnixEpoch,
		pub dateAdded: latest::MsSinceUnixEpoch,
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
		pub rating: Option<latest::PercentInteger>,
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
		pub dateImported: Option<latest::MsSinceUnixEpoch>,
		#[serde(default)]
		pub playCount: Option<u32>,
		#[serde(default)]
		pub plays: Option<Vec<latest::MsSinceUnixEpoch>>,
		#[serde(default)]
		pub playsImported: Option<Vec<latest::CountObject>>,
		#[serde(default)]
		pub skipCount: Option<u32>,
		#[serde(default)]
		pub skips: Option<Vec<latest::MsSinceUnixEpoch>>,
		#[serde(default)]
		pub skipsImported: Option<Vec<latest::CountObject>>,
		/// -100 to 100
		#[serde(default)]
		pub volume: Option<i8>,
	}

	#[derive(Deserialize, Clone, Debug)]
	#[serde(tag = "type")]
	pub enum TrackList {
		#[serde(rename = "playlist")]
		Playlist(Playlist),
		#[serde(rename = "folder")]
		Folder(latest::Folder),
		#[serde(rename = "special")]
		Special(latest::Special),
	}

	#[derive(Deserialize, Clone, Debug)]
	pub struct Playlist {
		pub id: TrackListID,
		pub name: String,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		pub description: Option<String>,
		#[serde(default, skip_serializing_if = "is_false")]
		pub liked: bool,
		#[serde(default, skip_serializing_if = "is_false")]
		pub disliked: bool,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		pub importedFrom: Option<String>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		pub originalId: Option<String>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		pub dateImported: Option<MsSinceUnixEpoch>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		pub dateCreated: Option<MsSinceUnixEpoch>,
		pub tracks: Vec<TrackID>,
	}
}

mod v2 {
	use crate::idvecmap::IdMap;
	use crate::library::Paths;
	use crate::migrate::{self, latest, queue_state_v0_and_v1, v1};
	use anyhow::Result;
	use linked_hash_map::LinkedHashMap;
	use serde::Deserialize;
	use std::borrow::Cow;
	use std::collections::HashMap;
	use std::sync::OnceLock;

	#[derive(Deserialize, Clone, Debug)]
	#[serde(deny_unknown_fields)]
	pub struct Library {
		pub tracks: LinkedHashMap<v1::TrackID, v1::Track>,
		pub trackLists: v1::TrackLists,
		pub v1PlayTime: Vec<v1::PlayTime>,
		pub playTime: Vec<v1::PlayTime>,
	}
	impl Library {
		pub fn upgrade<'a>(self, paths: &Paths) -> Result<latest::LatestLibrary<'a>> {
			let mut new_ids: HashMap<v1::TrackID, latest::TrackID> = HashMap::new();
			let mut tracks = IdMap::with_capacity(self.tracks.len());
			// Make sure IDs are generated first
			for (id, track) in self.tracks {
				let new_id = tracks.get_next_id();
				let removed = new_ids.insert(id, new_id);
				assert!(removed.is_none());
				tracks.try_insert(
					new_id,
					latest::Track {
						id: new_id,
						size: track.size,
						duration: track.duration,
						bitrate: track.bitrate,
						sampleRate: {
							let sample_rate = track.sampleRate.round();
							assert!(sample_rate.is_finite());
							assert!(sample_rate >= 0.0 && sample_rate <= u32::MAX as f64);
							sample_rate as u32
						},
						file: track.file,
						dateModified: track.dateModified,
						dateAdded: track.dateAdded,
						name: track.name,
						importedFrom: track.importedFrom,
						originalId: track.originalId,
						artist: track.artist,
						composer: track.composer,
						sortName: track.sortName,
						sortArtist: track.sortArtist,
						sortComposer: track.sortComposer,
						genre: track.genre,
						rating: track.rating,
						year: track.year,
						bpm: track.bpm,
						comments: track.comments,
						grouping: track.grouping,
						liked: track.liked,
						disliked: track.disliked,
						disabled: track.disabled,
						compilation: track.compilation,
						albumName: track.albumName,
						albumArtist: track.albumArtist,
						sortAlbumName: track.sortAlbumName,
						sortAlbumArtist: track.sortAlbumArtist,
						trackNum: track.trackNum,
						trackCount: track.trackCount,
						discNum: track.discNum,
						discCount: track.discCount,
						dateImported: track.dateImported,
						playCount: track.playCount,
						plays: track.plays,
						playsImported: track.playsImported,
						skipCount: track.skipCount,
						skips: track.skips,
						skipsImported: track.skipsImported,
						volume: track.volume,
					},
				);
			}
			let track_lists = self
				.trackLists
				.into_iter()
				.map(|(id, list)| {
					let list = match list {
						v1::TrackList::Playlist(playlist) => {
							latest::TrackList::Playlist(latest::Playlist {
								id: playlist.id,
								name: playlist.name,
								description: playlist.description,
								liked: playlist.liked,
								disliked: playlist.disliked,
								importedFrom: playlist.importedFrom,
								originalId: playlist.originalId,
								dateImported: playlist.dateImported,
								dateCreated: playlist.dateCreated,
								tracks: playlist
									.tracks
									.into_iter()
									.map(|id| new_ids[&id])
									.collect::<Vec<_>>(),
								item_ids: OnceLock::new(),
							})
						}
						v1::TrackList::Folder(folder) => latest::TrackList::Folder(folder),
						v1::TrackList::Special(special) => latest::TrackList::Special(special),
					};
					(id, list)
				})
				.collect();

			let library = latest::LatestLibrary {
				tracks: Cow::Owned(tracks),
				trackLists: Cow::Owned(track_lists),
				// Playtimes were incorrect and overwritten every relaunch, so nothing to keep
				playTimes: Cow::Owned(Vec::new()),
			};
			let library_file = migrate::LatestLibraryFile::V3(library);
			library_file.save(paths)?;
			queue_state_v0_and_v1::upgrade_file(&paths.queue_file, &new_ids);

			let library = match library_file {
				migrate::LatestLibraryFile::V3(library) => library,
			};
			Ok(library)
		}
	}
}

/// These formats had string track IDs
pub mod queue_state_v0_and_v1 {
	use crate::library_types::TrackID;

	use crate::queue_state as latest;
	use serde::Deserialize;
	use std::{collections::HashMap, fs};

	#[derive(Deserialize, Debug, Clone, Default)]
	pub struct QueueItemState {
		#[serde(rename = "qId")]
		pub q_id: i64,
		pub id: String,
		pub non_shuffle_pos: Option<u32>,
	}

	#[derive(Deserialize, Debug, Clone, Default)]
	pub struct QueueCurrentState {
		pub item: QueueItemState,
		pub from_auto_queue: bool,
	}

	#[derive(Deserialize, Debug, Clone, Default)]
	pub struct QueueState {
		#[serde(default)]
		pub past: Vec<QueueItemState>,
		#[serde(default)]
		pub current: Option<QueueCurrentState>,
		#[serde(default)]
		pub user_queue: Vec<QueueItemState>,
		#[serde(default)]
		pub auto_queue: Vec<QueueItemState>,
		#[serde(default)]
		pub last_qid: i64,
		#[serde(default)]
		pub shuffle: bool,
		#[serde(default)]
		pub repeat: bool,
	}

	#[derive(Deserialize, Debug, Clone)]
	#[serde(untagged)]
	enum DiskAutoQueueItem {
		Id(String),
		IdAndPos((String, u32)),
	}

	#[derive(Deserialize, Debug, Clone, Default)]
	struct DiskQueueState(
		Vec<String>,            // past
		Option<String>,         // current
		Vec<String>,            // user_queue
		Vec<DiskAutoQueueItem>, // auto_queue
		bool,                   // shuffle
		bool,                   // repeat
	);

	impl From<&QueueState> for DiskQueueState {
		fn from(value: &QueueState) -> Self {
			DiskQueueState(
				value.past.iter().map(|item| item.id.clone()).collect(),
				value
					.current
					.as_ref()
					.map(|current| current.item.id.clone()),
				value
					.user_queue
					.iter()
					.map(|item| item.id.clone())
					.collect(),
				value
					.auto_queue
					.iter()
					.map(|item| match item.non_shuffle_pos {
						Some(non_shuffle_pos) => {
							DiskAutoQueueItem::IdAndPos((item.id.clone(), non_shuffle_pos))
						}
						None => DiskAutoQueueItem::Id(item.id.clone()),
					})
					.collect(),
				value.shuffle,
				value.repeat,
			)
		}
	}

	impl From<DiskQueueState> for QueueState {
		fn from(value: DiskQueueState) -> Self {
			let DiskQueueState(past_ids, current_id, user_ids, auto_items, shuffle, repeat) = value;

			let mut next_qid: i64 = -1;
			let mut new_item = |id: String, non_shuffle_pos: Option<u32>| {
				next_qid += 1;
				QueueItemState {
					q_id: next_qid,
					id,
					non_shuffle_pos,
				}
			};

			let past = past_ids
				.into_iter()
				.map(|id| new_item(id, None))
				.collect::<Vec<_>>();
			let current = current_id.map(|id| QueueCurrentState {
				item: new_item(id, None),
				from_auto_queue: false,
			});
			let user_queue = user_ids
				.into_iter()
				.map(|id| new_item(id, None))
				.collect::<Vec<_>>();
			let auto_queue = auto_items
				.into_iter()
				.map(|item| match item {
					DiskAutoQueueItem::Id(id) => new_item(id, None),
					DiskAutoQueueItem::IdAndPos((id, non_shuffle_pos)) => {
						new_item(id, Some(non_shuffle_pos))
					}
				})
				.collect::<Vec<_>>();

			QueueState {
				past,
				current,
				user_queue,
				auto_queue,
				last_qid: next_qid,
				shuffle,
				repeat,
			}
		}
	}

	#[derive(Deserialize, Debug, Clone)]
	#[serde(untagged)]
	enum LegacyDiskQueueState {
		V0(QueueState),
		V1(DiskQueueState),
	}

	pub fn upgrade_file(
		file_path: &str,
		new_track_id_map: &HashMap<String, TrackID>,
	) -> Option<()> {
		let bytes = fs::read(file_path).ok()?;
		let queue_state = match serde_cbor::from_slice::<LegacyDiskQueueState>(&bytes) {
			Ok(LegacyDiskQueueState::V0(qs)) => qs,
			Ok(LegacyDiskQueueState::V1(qs)) => qs.into(),
			Err(_) => return None,
		};
		let new_queue_state = latest::QueueState {
			past: queue_state
				.past
				.into_iter()
				.filter_map(|item| {
					Some(latest::QueueItemState {
						q_id: item.q_id,
						id: *new_track_id_map.get(&item.id)?,
						non_shuffle_pos: item.non_shuffle_pos,
					})
				})
				.collect(),
			current: queue_state
				.current
				.map(|item| {
					Some(latest::QueueCurrentState {
						item: latest::QueueItemState {
							q_id: item.item.q_id,
							id: *new_track_id_map.get(&item.item.id)?,
							non_shuffle_pos: item.item.non_shuffle_pos,
						},
						from_auto_queue: item.from_auto_queue,
					})
				})
				.flatten(),
			user_queue: queue_state
				.user_queue
				.into_iter()
				.filter_map(|item| {
					Some(latest::QueueItemState {
						q_id: item.q_id,
						id: *new_track_id_map.get(&item.id)?,
						non_shuffle_pos: item.non_shuffle_pos,
					})
				})
				.collect(),
			auto_queue: queue_state
				.auto_queue
				.into_iter()
				.filter_map(|item| {
					Some(latest::QueueItemState {
						q_id: item.q_id,
						id: *new_track_id_map.get(&item.id)?,
						non_shuffle_pos: item.non_shuffle_pos,
					})
				})
				.collect(),
			last_qid: queue_state.last_qid,
			shuffle: queue_state.shuffle,
			repeat: queue_state.repeat,
		};
		new_queue_state.save(file_path).ok()?;
		Some(())
	}
}
