use crate::data_js::get_data;
use crate::get_now_timestamp;
use crate::library::Paths;
use crate::library_types::{ItemId, MsSinceUnixEpoch, TRACK_ID_MAP, Track, TrackID};
use anyhow::{Context, Result, bail};
use napi::bindgen_prelude::{ArrayBuffer, Buffer};
use std::fs;
use std::path::Path;

pub mod cover;
pub mod import;
mod md;
mod tag;

pub use tag::Tag;

#[napi(js_name = "get_track")]
#[allow(dead_code)]
pub fn get_track(id: String) -> Result<Track> {
	let data = get_data();
	let track = data.library.get_track(&id)?;
	Ok(track.clone())
}

#[napi(object)]
pub struct KeyedTrack {
	pub id: TrackID,
	pub track: Track,
}

#[napi(js_name = "get_track_by_item_id")]
#[allow(dead_code)]
pub fn get_track_by_item_id(item_id: ItemId) -> Result<KeyedTrack> {
	let data = get_data();
	let id_map = TRACK_ID_MAP.read().unwrap();
	let track_id = &id_map[item_id as usize];
	let track = data.library.get_track(&track_id)?;
	Ok(KeyedTrack {
		id: track_id.clone(),
		track: track.clone(),
	})
}

#[napi(js_name = "get_track_ids")]
#[allow(dead_code)]
pub fn get_track_ids(item_ids: Vec<ItemId>) -> Vec<TrackID> {
	let id_map = TRACK_ID_MAP.read().unwrap();
	let track_ids = item_ids.into_iter().map(|item_id| {
		let track_id = &id_map[item_id as usize];
		track_id.clone()
	});
	track_ids.collect()
}

#[napi(js_name = "track_exists")]
#[allow(dead_code)]
pub fn track_exists(id: String) -> bool {
	let data = get_data();
	let tracks = &data.library.get_tracks();
	tracks.contains_key(&id)
}

#[napi(js_name = "add_play")]
#[allow(dead_code)]
pub fn add_play(track_id: String) -> Result<()> {
	let mut data = get_data();
	let track = data.library.get_track_mut(&track_id)?;
	let timestamp = get_now_timestamp();
	match &mut track.plays {
		None => track.plays = Some(vec![timestamp]),
		Some(plays) => plays.push(timestamp),
	}
	match &mut track.playCount {
		None => track.playCount = Some(1),
		Some(play_count) => *play_count += 1,
	}
	Ok(())
}

#[napi(js_name = "add_skip")]
#[allow(dead_code)]
pub fn add_skip(track_id: String) -> Result<()> {
	let mut data = get_data();
	let track = data.library.get_track_mut(&track_id)?;
	let timestamp = get_now_timestamp();
	match &mut track.skips {
		None => track.skips = Some(vec![timestamp]),
		Some(skips) => skips.push(timestamp),
	}
	match &mut track.skipCount {
		None => track.skipCount = Some(1),
		Some(skip_count) => *skip_count += 1,
	}
	Ok(())
}

#[napi(js_name = "add_play_time")]
#[allow(dead_code)]
pub fn add_play_time(id: TrackID, start: MsSinceUnixEpoch, dur_ms: i64) -> Result<()> {
	let mut data = get_data();
	let tracks = data.library.get_tracks();
	tracks.get(&id).context("Track ID not found")?;
	data.library.playTime.push((id, start, dur_ms));
	Ok(())
}

fn sanitize_filename(input: &String) -> String {
	let mut string = input.replace('/', "_");
	string = string.replace('?', "_");
	string = string.replace('<', "_");
	string = string.replace('>', "_");
	string = string.replace('\\', "_");
	string = string.replace(':', "_");
	string = string.replace('*', "_");
	string = string.replace('\"', "_");
	// prevent control characters:
	string = string.replace("0x", "__");
	// Filenames can be max 255 bytes. We use 230 to give
	// margin for the fileNum and file extension.
	string.truncate(230);
	return string;
}

pub fn generate_filename(paths: &Paths, artist: &str, title: &str, ext: &str) -> String {
	let beginning = artist.to_owned() + " - " + title;
	let beginning = sanitize_filename(&beginning);

	let mut file_num: u32 = 1;
	let mut filename = beginning.clone() + "." + ext;
	for i in 0..9999 {
		if i == 1000 {
			panic!("Already got 500 files with that artist and title")
		}
		let full_path = paths.get_track_file_path(&filename);
		if full_path.exists() {
			file_num += 1;
			filename = beginning.clone() + " " + file_num.to_string().as_str() + "." + ext;
		} else {
			break;
		}
	}
	return filename;
}

#[napi(js_name = "import_file")]
#[allow(dead_code)]
pub fn import_file(path: String, now: MsSinceUnixEpoch) -> Result<()> {
	let mut data = get_data();
	let id = data.library.generate_id();
	let track = import::import(&data, Path::new(&path), now)?;
	data.library.insert_track(id, track);
	Ok(())
}

#[napi(js_name = "load_tags")]
#[allow(dead_code)]
pub fn load_tags(track_id: String) -> Result<()> {
	let data = &mut *get_data();
	data.current_tag = None;
	let track = data
		.library
		.get_track_mut(&track_id)
		.context("Could not load tags")?;

	let path = data.paths.get_track_file_path(&track.file);
	let tag = Tag::read_from_path(&path).context("Could not load tags")?;
	data.current_tag = Some(tag);
	Ok(())
}

#[napi(object)]
pub struct JsImage {
	pub index: i64,
	pub total_images: i64,
	pub mime_type: String,
	pub data: Buffer,
}

#[napi(js_name = "get_image")]
#[allow(dead_code)]
pub fn get_image(index: u32) -> Result<Option<JsImage>> {
	let data = get_data();

	let tag = match &data.current_tag {
		Some(tag) => tag,
		None => bail!("Could not load image: No tag loaded"),
	};
	let img = match tag
		.get_image_ref(index as usize)
		.context("Could not load image")?
	{
		Some(image) => image,
		None => return Ok(None),
	};

	let js_image = JsImage {
		index: img.index,
		total_images: img.total_images,
		mime_type: img.mime_type.to_string(),
		data: img.data.into(),
	};
	Ok(Some(js_image))
}

#[napi(js_name = "set_image")]
#[allow(dead_code)]
pub fn set_image(index: u32, path: String) -> Result<()> {
	let mut data = get_data();
	let tag = match &mut data.current_tag {
		Some(tag) => tag,
		None => bail!("No tag loaded"),
	};
	let new_bytes = fs::read(&path).context("Error reading that file")?;
	tag.set_image(index as usize, new_bytes)?;
	Ok(())
}

#[napi(js_name = "set_image_data")]
#[allow(dead_code)]
pub fn set_image_data(index: u32, bytes: ArrayBuffer) -> Result<()> {
	let mut data = get_data();
	let tag = match &mut data.current_tag {
		Some(tag) => tag,
		None => bail!("No tag loaded"),
	};
	tag.set_image(index as usize, bytes.to_vec())?;
	Ok(())
}

#[napi(js_name = "remove_image")]
#[allow(dead_code)]
pub fn remove_image(index: u32) -> () {
	let mut data = get_data();
	match data.current_tag {
		Some(ref mut tag) => {
			tag.remove_image(index as usize);
		}
		None => {}
	};
}

#[napi(js_name = "update_track_info")]
#[allow(dead_code)]
pub fn update_track_info(track_id: String, info: md::TrackMD) -> Result<()> {
	let data = &mut *get_data();
	let track = data.library.get_track_mut(&track_id)?;

	let tag = match &mut data.current_tag {
		Some(tag) => tag,
		None => bail!("No tag loaded"),
	};
	md::update_track_info(&data.paths, track, tag, info)?;

	Ok(())
}
