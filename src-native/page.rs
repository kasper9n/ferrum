#[cfg(feature = "napi-rs")]
use crate::data::Data;
use crate::filter::{FilterTerm, filter};
use crate::library_types::{ItemId, Library, TrackList};
use crate::sort::sort;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Mutex;

/// We don't keep the filter_terms in here, so that we can cache the pre-filtered page
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Deserialize, Clone, Type, Eq, PartialEq)]
pub struct TracksPageOptions {
	pub playlist_id: String,
	pub sort_key: String,
	pub sort_desc: bool,
	pub group_album_tracks: bool,
	pub library_cache_key: u32,
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
pub fn get_tracks_page_js(
	options: TracksPageOptions,
	filter_terms: Vec<FilterTerm>,
) -> Result<TracksPage> {
	let data = Data::get_blocking();
	Ok(get_tracks_page(options, filter_terms, &data.library)?)
}

static LAST_SORT: Mutex<Option<(TracksPageOptions, Vec<ItemId>)>> = Mutex::new(None);

fn sort_cached(options: &TracksPageOptions, library: &Library) -> Result<Vec<ItemId>> {
	if let Some((cached_options, ids)) = LAST_SORT.lock().unwrap().as_ref() {
		if cached_options == options {
			return Ok(ids.clone());
		}
	}

	let ids = sort(options.clone(), library)?;

	let mut last_sort = LAST_SORT.lock().unwrap();
	*last_sort = Some((options.clone(), ids.clone()));

	Ok(ids)
}

pub fn get_tracks_page(
	options: TracksPageOptions,
	filter_terms: Vec<FilterTerm>,
	library: &Library,
) -> anyhow::Result<TracksPage> {
	let tracklist = library.get_tracklist(&options.playlist_id)?;
	let item_ids = sort_cached(&options, &library)?;
	let tracklist_length = item_ids.len();
	let item_ids = filter(item_ids, filter_terms, &library);
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
