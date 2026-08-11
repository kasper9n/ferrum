#[cfg(feature = "napi-rs")]
use crate::data_js::get_data;
use crate::filter::filter;
use crate::library_types::{ItemId, Library, TrackList};
use crate::sort::sort;
#[cfg(feature = "napi-rs")]
use napi::Result;
use serde::{Deserialize, Serialize};
use specta::Type;

#[cfg_attr(feature = "napi", napi(object))]
#[derive(Deserialize, Clone, Type)]
pub struct TracksPageOptions {
	pub playlist_id: String,
	pub sort_key: String,
	pub sort_desc: bool,
	pub filter_query: String,
	pub group_album_tracks: bool,
}

#[cfg_attr(feature = "napi", napi(object))]
#[derive(Serialize, Type)]
pub struct TracksPage {
	pub playlist_kind: String,
	pub playlist_name: String,
	pub playlist_description: Option<String>,
	pub playlist_length: u32,
	pub item_ids: Vec<ItemId>,
}

#[cfg(feature = "napi-rs")]
#[cfg_attr(feature = "napi", napi(js_name = "get_tracks_page"))]
#[allow(dead_code)]
pub fn get_tracks_page(options: TracksPageOptions) -> Result<TracksPage> {
	let data = get_data();
	Ok(get_tracks_page_from_library(options, &data.library)?)
}

pub fn get_tracks_page_from_library(
	options: TracksPageOptions,
	library: &Library,
) -> anyhow::Result<TracksPage> {
	let tracklist = library.get_tracklist(&options.playlist_id)?;
	let item_ids = sort(options.clone(), &library)?;
	let tracklist_length = item_ids.len();
	let item_ids = filter(item_ids, options.filter_query, &library);
	let track_page = match tracklist {
		TrackList::Playlist(playlist) => TracksPage {
			playlist_kind: tracklist.kind().to_string(),
			playlist_name: playlist.name.clone(),
			playlist_description: playlist.description.clone(),
			playlist_length: tracklist_length as u32,
			item_ids,
		},
		TrackList::Folder(folder) => TracksPage {
			playlist_kind: tracklist.kind().to_string(),
			playlist_name: folder.name.clone(),
			playlist_description: folder.description.clone(),
			playlist_length: tracklist_length as u32,
			item_ids,
		},
		TrackList::Special(special) => TracksPage {
			playlist_kind: tracklist.kind().to_string(),
			playlist_name: special.name.to_string(),
			playlist_description: None,
			playlist_length: tracklist_length as u32,
			item_ids,
		},
	};
	Ok(track_page)
}
