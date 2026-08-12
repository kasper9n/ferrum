#![allow(non_snake_case)]

use super::{Tag, generate_filename};
use crate::library::Paths;
use crate::library_types::Track;
use crate::{get_now_timestamp, str_to_option};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "napi-rs", napi(object))]
pub struct TrackMD {
	pub name: String,
	pub artist: String,
	pub albumName: String,
	pub albumArtist: String,
	pub composer: String,
	pub grouping: String,
	pub genre: String,
	pub year: String,
	pub trackNum: String,
	pub trackCount: String,
	pub discNum: String,
	pub discCount: String,
	pub bpm: String,
	// compilation: String,
	// rating: String,
	// liked: String,
	// playCount: String,
	pub comments: String,
}

pub fn update_track_info(
	paths: &Paths,
	track: &mut Track,
	tag: &mut Tag,
	new_info: TrackMD,
) -> Result<()> {
	let old_path = paths.get_track_file_path(&track.file);
	if !old_path.exists() {
		bail!("File does not exist: {}", track.file);
	}
	let ext = old_path.extension().unwrap_or_default().to_str().unwrap();

	// name
	match new_info.name.as_ref() {
		"" => tag.remove_title(),
		value => tag.set_title(value),
	};
	let new_name = new_info.name.clone();

	// artists
	match new_info.artist.as_ref() {
		"" => tag.remove_artists(),
		value => tag.set_artist(value),
	};
	let new_artist = new_info.artist.clone();

	// album_name
	match new_info.albumName.as_ref() {
		"" => tag.remove_album(),
		value => tag.set_album(value),
	};
	let new_album_name = str_to_option(new_info.albumName);

	// album_artist
	match new_info.albumArtist.as_ref() {
		"" => tag.remove_album_artists(),
		value => tag.set_album_artist(value),
	};
	let new_album_artist = str_to_option(new_info.albumArtist);

	// composer
	match new_info.composer.as_ref() {
		"" => tag.remove_composers(),
		value => tag.set_composer(value),
	};
	let new_composer = str_to_option(new_info.composer);

	// grouping
	match new_info.grouping.as_ref() {
		"" => tag.remove_groupings(),
		value => tag.set_grouping(value),
	};
	let new_grouping = str_to_option(new_info.grouping);

	// genre
	match new_info.genre.as_ref() {
		"" => tag.remove_genres(),
		value => tag.set_genre(value),
	};
	let new_genre = str_to_option(new_info.genre);

	// year
	let new_year_i32 = match new_info.year.as_ref() {
		"" => None,
		value => Some(value.parse().context("Invalid year")?),
	};
	let new_year_i64 = new_year_i32.map(i64::from);
	match new_year_i32 {
		None => tag.remove_year(),
		Some(value) => tag.set_year(value),
	};

	// track_number, track_count
	let new_track_number: Option<u32> = match new_info.trackNum.as_ref() {
		"" => None,
		value => Some(value.parse().context("Invalid track number")?),
	};
	let new_track_count: Option<u32> = match new_info.trackCount.as_ref() {
		"" => None,
		value => Some(value.parse().context("Invalid track count")?),
	};
	tag.set_track_info(new_track_number, new_track_count);

	// disc_number, disc_count
	let new_disc_number: Option<u32> = match new_info.discNum.as_ref() {
		"" => None,
		value => Some(value.parse().context("Invalid disc number")?),
	};
	let new_disc_count: Option<u32> = match new_info.discCount.as_ref() {
		"" => None,
		value => Some(value.parse().context("Invalid disc count")?),
	};
	tag.set_disc_info(new_disc_number, new_disc_count);

	let new_bpm: Option<u16> = match new_info.bpm.as_ref() {
		"" => None,
		value => Some(value.parse().context("Invalid bpm")?),
	};
	match new_bpm {
		None => tag.remove_bpm(),
		Some(value) => tag.set_bpm(value),
	};

	// comment
	match new_info.comments.as_ref() {
		"" => tag.remove_comments(),
		value => tag.set_comment(value),
	};
	let new_comments = str_to_option(new_info.comments);

	// save tag
	tag.write_to_path(&old_path).context("Failed to save tag")?;

	// move file
	if new_name != track.name || new_artist != track.artist {
		let new_filename = generate_filename(&paths, &new_artist, &new_name, &ext);
		let new_path = paths.get_track_file_path(&new_filename);
		match fs::rename(old_path, new_path) {
			Ok(_) => {
				track.file = new_filename;
			}
			Err(_) => {}
		}
	}

	track.name = new_name;
	track.artist = new_artist;
	track.albumName = new_album_name;
	track.albumArtist = new_album_artist;
	track.composer = new_composer;
	track.grouping = new_grouping;
	track.genre = new_genre;
	track.year = new_year_i64;
	track.trackNum = new_track_number;
	track.trackCount = new_track_count;
	track.discNum = new_disc_number;
	track.discCount = new_disc_count;
	track.bpm = new_bpm.map(|n| n.into());
	track.comments = new_comments;
	track.dateModified = get_now_timestamp();

	Ok(())
}
