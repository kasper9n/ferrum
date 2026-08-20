use crate::data::Data;
use crate::db::TrackListKind;
use crate::filter::{FilterTerm, filter};
use crate::library_types::new_item_ids_from_track_ids;
use crate::library_types::{ItemId, Library, TrackList};
use crate::sort::sort;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::Connection;

#[cfg_attr(feature = "napi", napi(object))]
#[derive(Deserialize, Clone, Type)]
pub struct TracksPageOptions {
	pub playlist_id: String,
	pub sort_key: String,
	pub sort_desc: bool,
	pub filter_terms: Vec<FilterTerm>,
	pub group_album_tracks: bool,
}

#[cfg_attr(feature = "napi", napi(object))]
#[derive(Serialize, Type)]
pub struct TracksPage {
	pub playlist_kind: String,
	pub playlist_name: String,
	pub playlist_description: String,
	pub playlist_length: u32,
	pub item_ids: Vec<ItemId>,
}

#[derive(Debug, sqlx::FromRow)]
struct TrackListPage {
	kind: TrackListKind,
	name: String,
	description: String,
}

// returns (column_name, is_text)
fn to_sql_sort_key(sort_key: &str) -> (&'static str, bool) {
	match sort_key {
		"albumName" => ("album_title", true),
		"albumArtist" => ("album_artist", true),
		"artist" => ("artist", true),
		"bpm" => ("bpm", false),
		"comments" => ("comments", true),
		"composer" => ("composer", true),
		"dateAdded" => ("added_at", false),
		"duration" => ("duration_s", false),
		"genre" => ("genre", true),
		"grouping" => ("grouping", true),
		"name" => ("title", true),
		"playCount" => ("play_count", false),
		"skipCount" => ("skip_count", false),
		"year" => ("year", false),
		sort_key => panic!("Invalid sort key {sort_key}"),
	}
}

#[derive(sqlx::FromRow)]
struct AllTrackRow {
	id: String,
	title: String,
	artist: String,
	composer: Option<String>,
	genre: Option<String>,
	comments: Option<String>,
	grouping: Option<String>,
	album_title: Option<String>,
	album_artist: Option<String>,
	added_at: i64,
	duration_s: f64,
	bpm: Option<f64>,
	play_count: u64,
	skip_count: u64,
	year: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct PlaylistTrackRow {
	track_list_id: String,
	item_pos: u64,
	id: String,
	title: String,
	artist: String,
	composer: Option<String>,
	genre: Option<String>,
	comments: Option<String>,
	grouping: Option<String>,
	album_title: Option<String>,
	album_artist: Option<String>,
	added_at: i64,
	duration_s: f64,
	bpm: Option<f64>,
	play_count: u64,
	skip_count: u64,
	year: Option<i64>,
}

#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi", napi(js_name = "get_tracks_page"))]
#[allow(dead_code)]
pub async fn get_tracks_page_js(options: TracksPageOptions) -> Result<TracksPage> {
	get_tracks_page(options).await
}

pub async fn get_tracks_page(options: TracksPageOptions) -> Result<TracksPage> {
	let mut data = Data::get_async().await;
	let mut tx = data.db.begin().await?;

	let start_time = std::time::Instant::now();
	let track_list: TrackListPage = sqlx::query_as(
		"SELECT kind, name, description
		FROM track_lists
		WHERE id = ?",
	)
	.bind(&options.playlist_id)
	.fetch_one(&mut *tx)
	.await?;

	// todo: sort, filter

	tx.commit().await?;

	let track_ids = vec![];

	// todo: remove
	let item_ids = new_item_ids_from_track_ids(&track_ids);

	println!(
		"get_tracks_page took {:?}, {} results",
		start_time.elapsed(),
		track_ids.len()
	);
	let tracks_page = TracksPage {
		playlist_kind: track_list.kind.to_string(),
		playlist_name: track_list.name,
		playlist_description: track_list.description,
		playlist_length: track_ids.len().try_into().unwrap(),
		item_ids,
	};
	Ok(tracks_page)
}

#[cfg(test)]
mod tests {
	use crate::{
		data::Data,
		library_types::SpecialTrackListName,
		page::{TracksPageOptions, get_tracks_page},
	};
	use std::path::PathBuf;

	#[tokio::test]
	async fn test_get_tracks_page() -> anyhow::Result<()> {
		let library_path = PathBuf::from("./src-native/appdata/Library big");
		Data::load(true, None, Some(library_path.to_string_lossy().to_string()))
			.await
			.unwrap();
		let result = get_tracks_page(TracksPageOptions {
			playlist_id: SpecialTrackListName::Root.get_id().to_string(),
			sort_key: "name".to_string(),
			sort_desc: false,
			filter_terms: vec![],
			group_album_tracks: false,
		})
		.await?;

		println!("result: {:#?}", result.item_ids.len());

		Ok(())
	}
}
