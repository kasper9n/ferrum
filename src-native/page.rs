use crate::data::Data;
use crate::db::TrackListKind;
use crate::filter::{FilterTerm, insert_queued_track_ngrams};
use crate::library_types::{ItemId, new_item_ids_from_track_ids};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{AssertSqlSafe, Connection};

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

// enum FilterArg {
// 	Text(String),
// 	Integer(i64),
// 	Real(f64),
// }

// fn add_text_filter(sql: &mut String, args: &mut Vec<FilterArg>, field: i32, literal: &str) {
// 	sql.push_str(
// 		" AND EXISTS (
// 			SELECT 1
// 			FROM search_ngrams sn
// 			WHERE sn.track_id = t.id
// 			  AND sn.field = ?
// 			  AND sn.ngram = ?
// 		)",
// 	);

// 	args.push(FilterArg::Integer(field as i64));
// 	args.push(FilterArg::Text(literal.to_owned()));
// }

pub async fn get_tracks_page(options: TracksPageOptions) -> Result<TracksPage> {
	{
		insert_queued_track_ngrams().await?;
	}

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

	let sql = String::from(
		"
		WITH term1 AS (
			SELECT track_id
			FROM search_ngrams
			WHERE ngram IN ('dev', 'evo', 'vot', 'oti', 'tio', 'ion')
				AND field IN (0, 1, 2)
			 	AND is_normalised = 0
			GROUP BY track_id, field
			HAVING COUNT(DISTINCT ngram) = 6
		),
		term2 AS (
			SELECT track_id
			FROM search_ngrams
			WHERE ngram IN ('tri', 'ris', 'ist', 'sta', 'tam')
				AND field IN (0, 1, 2)
			 	AND is_normalised = 0
			GROUP BY track_id, field
			HAVING COUNT(DISTINCT ngram) = 5
		)
		SELECT track_id FROM term1
		INTERSECT
		SELECT track_id FROM term2;
		",
	);
	// let mut sql = String::from(
	// 	"SELECT pt.track_id
	// 	FROM playlist_tracks pt
	// 	JOIN tracks t ON t.id = pt.track_id
	// 	WHERE pt.track_list_id = ?",
	// );
	// let mut where_clauses = Vec::new();
	let mut args = sqlx::sqlite::SqliteArguments::default();

	// for term in options.filter_terms.iter().filter(|t| !t.is_whitespace()) {
	// 	match term.field {
	// 		// Some(Field::Title) => add_text_filter(&mut sql, &mut args, 0, &term.literal),
	// 		// Some(Field::Artist) => add_text_filter(&mut sql, &mut args, 1, &term.literal),
	// 		// Some(Field::Album) => add_text_filter(&mut sql, &mut args, 2, &term.literal),
	// 		// Some(Field::AlbumArtist) => add_text_filter(&mut sql, &mut args, 3, &term.literal),
	// 		// Some(Field::Comments) => add_text_filter(&mut sql, &mut args, 4, &term.literal),
	// 		// Some(Field::Genre) => add_text_filter(&mut sql, &mut args, 5, &term.literal),
	// 		// Some(Field::Composer) => add_text_filter(&mut sql, &mut args, 6, &term.literal),
	// 		// Some(Field::Group) => add_text_filter(&mut sql, &mut args, 7, &term.literal),
	// 		None => {
	// 			where_clauses.push(
	// 				"EXISTS (
	// 					SELECT 1
	// 					FROM search_ngrams sn
	// 					WHERE sn.track_id = t.id
	// 					  AND sn.field BETWEEN 0 AND 7
	// 					  AND sn.ngram = ?
	// 				)",
	// 			);
	// 			args.add(&term.literal);
	// 		}
	// 		_ => todo!(),
	// 	}
	// }

	// sql.push_str(" ORDER BY ");

	// if options.group_album_tracks {
	// 	sql.push_str(
	// 		"t.album_artist COLLATE NOCASE,
	// 		 t.album_title COLLATE NOCASE,
	// 		 t.disc_num,
	// 		 t.track_num,
	// 		 ",
	// 	);
	// }

	// let sort_column = match options.sort_key.as_str() {
	// 	"title" => "t.title",
	// 	"artist" => "t.artist",
	// 	"album" => "t.album_title",
	// 	"album_artist" => "t.album_artist",
	// 	"comments" => "t.comments",
	// 	"genre" => "t.genre",
	// 	"composer" => "t.composer",
	// 	"group" => "t.grouping",
	// 	_ => "pt.item_pos",
	// };

	// sql.push_str(sort_column);
	// sql.push_str(if options.sort_desc { " DESC" } else { " ASC" });

	// // Stable ordering.
	// sql.push_str(", pt.item_pos ASC");

	let track_ids: Vec<i64> = sqlx::query_scalar_with(AssertSqlSafe(sql), args)
		.fetch_all(&mut *tx)
		.await
		.context("Failed to select page track_ids")?;

	println!(
		"get_tracks_page took {:?}, {} results",
		start_time.elapsed(),
		track_ids.len()
	);

	let text_ids: Vec<String> = sqlx::query_scalar(
		"SELECT text_id FROM tracks WHERE id IN (SELECT value FROM json_each(?))",
	)
	.bind(serde_json::to_string(&track_ids)?)
	.fetch_all(&mut *tx)
	.await?;
	let item_ids = new_item_ids_from_track_ids(&text_ids);

	tx.commit().await?;

	Ok(TracksPage {
		playlist_kind: track_list.kind.to_string(),
		playlist_name: track_list.name,
		playlist_description: track_list.description,
		playlist_length: track_ids.len().try_into().unwrap(),
		item_ids,
	})
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
