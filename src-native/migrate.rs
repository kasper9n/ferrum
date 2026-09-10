#![allow(non_snake_case)]

pub(self) use crate::library_types as latest;
use crate::library_types::LatestLibrary;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs::File, io::Read};

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

/// For serialization, since we don't need to serialize old formats
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "version", deny_unknown_fields)]
pub enum LatestLibraryFile<'a> {
	#[serde(rename = "3")]
	V3(LatestLibrary<'a>),
}

pub fn upgrade<'a>(versioned_library: LibraryFile<'a>) -> LatestLibrary<'a> {
	match versioned_library {
		LibraryFile::V1(v1) => v1.upgrade().upgrade(),
		LibraryFile::V2(v2) => v2.upgrade(),
		LibraryFile::V3(v3) => v3,
	}
}

pub fn parse_old_version_library_json(library_file: &mut File) -> Result<LibraryFile<'_>> {
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

	let versioned_library: LibraryFile =
		serde_json::from_value(value).context("Error parsing library file")?;
	Ok(versioned_library)
}

mod v1 {
	use crate::migrate::{latest, v2};
	use linked_hash_map::LinkedHashMap;
	use serde::Deserialize;

	#[derive(Deserialize, Clone, Debug)]
	#[serde(deny_unknown_fields)]
	pub struct Library {
		tracks: LinkedHashMap<TrackID, latest::Track>,
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
	pub type TrackLists = LinkedHashMap<TrackListID, latest::TrackList>;

	/// (track id, start time, duration)
	pub type PlayTime = (TrackID, MsSinceUnixEpoch, i64);
}

mod v2 {
	use crate::migrate::{latest, v1};
	use linked_hash_map::LinkedHashMap;
	use serde::Deserialize;
	use std::borrow::Cow;

	#[derive(Deserialize, Clone, Debug)]
	#[serde(deny_unknown_fields)]
	pub struct Library {
		pub tracks: LinkedHashMap<v1::TrackID, latest::Track>,
		pub trackLists: v1::TrackLists,
		pub v1PlayTime: Vec<v1::PlayTime>,
		pub playTime: Vec<v1::PlayTime>,
	}
	impl Library {
		pub fn upgrade<'a>(self) -> latest::LatestLibrary<'a> {
			latest::LatestLibrary {
				tracks: Cow::Owned(self.tracks),
				trackLists: Cow::Owned(self.trackLists),
				v1PlayTime: Cow::Owned(self.v1PlayTime),
				playTime: Cow::Owned(self.playTime),
			}
		}
	}
}
