#[cfg(feature = "napi-rs")]
use crate::data::Data;
use crate::db::TrackListKind;
use crate::filter::filter;
use crate::library_types::ItemId;
#[cfg(feature = "napi-rs")]
use crate::library_types::new_item_ids_from_track_ids;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
#[cfg(feature = "napi-rs")]
use sqlx::{Arguments, AssertSqlSafe, Connection};

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

#[cfg(feature = "napi-rs")]
#[cfg_attr(feature = "napi", napi(js_name = "get_tracks_page"))]
#[allow(dead_code)]
pub async fn get_tracks_page(options: TracksPageOptions) -> Result<TracksPage> {
	let start_time = std::time::Instant::now();
	let mut data = Data::get_async().await;
	let mut tx = data.db.begin().await?;
	let track_list: TrackListPage = sqlx::query_as(
		"SELECT kind, name, description
		FROM track_lists
		WHERE id = ?",
	)
	.bind(&options.playlist_id)
	.fetch_one(&mut *tx)
	.await?;

	// todo: FTS match OR exact match on 	year || bpm || play_count || skip_count

	let mut where_clauses = Vec::new();
	let mut args = sqlx::sqlite::SqliteArguments::default();

	println!("{track_list:?}");
	if track_list.kind == TrackListKind::Playlist {
		where_clauses.push("playlist_tracks.track_list_id = ?");
		args.add(&options.playlist_id).unwrap();
	}

	if options.filter_query.trim() != "" {
		where_clauses.push("tracks_fts MATCH ?");
		args.add(&options.filter_query).unwrap();
	}

	let direction = match options.sort_desc {
		true => "DESC",
		false => "ASC",
	};
	let mut order_by_clauses = Vec::new();
	match options.sort_key.as_str() {
		"index" => match track_list.kind {
			TrackListKind::Playlist => {
				order_by_clauses.push(format!("playlist_tracks.item_pos {direction}"))
			}
			TrackListKind::Folder => todo!(),
			TrackListKind::Special => order_by_clauses.push(format!("tracks.added_at {direction}")),
		},
		sort_key => {
			// TEXT columns should have empty values sorted last
			let (sql_sort_key, is_text_col) = to_sql_sort_key(sort_key);
			if is_text_col {
				order_by_clauses.push(format!(
					"CASE WHEN tracks.{sql_sort_key} IS NULL OR tracks.{sql_sort_key} = '' \
             THEN 1 ELSE 0 END ASC"
				));
			}
			match track_list.kind {
				TrackListKind::Playlist => {
					order_by_clauses.push(format!("tracks.{sql_sort_key} {direction}"));
					order_by_clauses.push("playlist_tracks.item_pos ASC".to_string());
				}
				TrackListKind::Folder => todo!(),
				TrackListKind::Special => {
					order_by_clauses.push(format!("tracks.{sql_sort_key} {direction}"));
					order_by_clauses.push("tracks.added_at ASC".to_string());
				}
			}
		}
	};
	let order_by = order_by_clauses.join(", ");

	let where_clause = match where_clauses.len() > 0 {
		true => format!("WHERE {}", where_clauses.join(" AND ")),
		false => "".to_string(),
	};
	let track_ids_sql = match track_list.kind {
		TrackListKind::Playlist => format!(
			"SELECT tracks.id
			FROM playlist_tracks
			JOIN tracks ON tracks.id = playlist_tracks.track_id
			JOIN tracks_fts ON tracks_fts.rowid = tracks.rowid
			{where_clause}
			ORDER BY {order_by}",
		),
		TrackListKind::Folder => todo!(),
		TrackListKind::Special => format!(
			"SELECT tracks.id
			FROM tracks
			JOIN tracks_fts ON tracks_fts.rowid = tracks.rowid
			{where_clause}
			ORDER BY {order_by}",
		),
	};
	println!("{track_ids_sql}");

	let track_ids: Vec<String> = sqlx::query_scalar_with(AssertSqlSafe(track_ids_sql), args)
		.fetch_all(&mut *tx)
		.await?;
	println!("{track_ids:?}");

	tx.commit().await?;

	// let track_ids = filter(track_ids, options.filter_query);

	println!("get_tracks_page took {:?}", start_time.elapsed());

	// todo: temporary workaround
	let item_ids = new_item_ids_from_track_ids(&track_ids);

	let tracks_page = TracksPage {
		playlist_kind: track_list.kind.to_string(),
		playlist_name: track_list.name,
		playlist_description: track_list.description,
		playlist_length: track_ids.len().try_into().unwrap(),
		item_ids,
	};
	Ok(tracks_page)
}
