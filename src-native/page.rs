use crate::data::Data;
use crate::db::TrackListKind;
use crate::library::Paths;
use crate::library_types::new_item_ids_from_track_ids;
use crate::library_types::{ItemId, SpecialTrackListName};
use anyhow::Result;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::Connection;
use sqlx::Row;
use std::path::Path;
use std::time::Instant;
use tantivy::collector::{Collector, TopDocs};
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::tokenizer::NgramTokenizer;
use tantivy::{DocAddress, Index, IndexReader, IndexWriter, Searcher, doc};
use tantivy::{Order, schema::*};

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

struct TantivyFields {
	track_id: Field,
	playlist_id: Field,
	playlist_pos: Field,
	added_at: Field,
	duration_s: Field,
	bpm: Field,
	play_count: Field,
	skip_count: Field,
	year: Field,
	title: Field,
	title_sort: Field,
	artist: Field,
	artist_sort: Field,
	composer: Field,
	composer_sort: Field,
	genre: Field,
	genre_sort: Field,
	comments: Field,
	comments_sort: Field,
	grouping: Field,
	grouping_sort: Field,
	album_title: Field,
	album_title_sort: Field,
	album_artist: Field,
	album_artist_sort: Field,
}

fn build_tantivy_schema() -> (Schema, TantivyFields) {
	let mut builder = Schema::builder();

	// todo: remove STORED, if possible

	// IDs
	let track_id = builder.add_text_field("track_id", STRING | STORED);
	let playlist_id = builder.add_text_field("playlist_id", STRING);

	// Numeric sorts (need FAST)
	let playlist_pos = builder.add_u64_field("playlist_pos", FAST);
	let added_at = builder.add_i64_field("added_at", FAST);
	let duration_s = builder.add_f64_field("duration_s", FAST);
	let bpm = builder.add_f64_field("bpm", FAST);
	let play_count = builder.add_u64_field("play_count", FAST);
	let skip_count = builder.add_u64_field("skip_count", FAST);
	let year = builder.add_i64_field("year", FAST);

	// Searchable text: 1-, 2-, and 3-character inner ngrams.
	let subword_options = TextOptions::default().set_indexing_options(
		TextFieldIndexing::default()
			.set_tokenizer("subword")
			.set_index_option(IndexRecordOption::WithFreqsAndPositions),
	);

	// Searchable text (TEXT = tokenized)
	let title = builder.add_text_field("title", subword_options.clone());
	let artist = builder.add_text_field("artist", subword_options.clone());
	let composer = builder.add_text_field("composer", subword_options.clone());
	let genre = builder.add_text_field("genre", subword_options.clone());
	let comments = builder.add_text_field("comments", subword_options.clone());
	let grouping = builder.add_text_field("grouping", subword_options.clone());
	let album_title = builder.add_text_field("album_title", subword_options.clone());
	let album_artist = builder.add_text_field("album_artist", subword_options);

	// Sortable text (STRING | FAST = lexicographic sort, not tokenized)
	let title_sort = builder.add_text_field("title_sort", STRING | FAST);
	let artist_sort = builder.add_text_field("artist_sort", STRING | FAST);
	let composer_sort = builder.add_text_field("composer_sort", STRING | FAST);
	let genre_sort = builder.add_text_field("genre_sort", STRING | FAST);
	let comments_sort = builder.add_text_field("comments_sort", STRING | FAST);
	let grouping_sort = builder.add_text_field("grouping_sort", STRING | FAST);
	let album_title_sort = builder.add_text_field("album_title_sort", STRING | FAST);
	let album_artist_sort = builder.add_text_field("album_artist_sort", STRING | FAST);

	let schema = builder.build();
	let fields = TantivyFields {
		track_id,
		playlist_id,
		playlist_pos,
		added_at,
		duration_s,
		bpm,
		play_count,
		skip_count,
		year,
		title,
		title_sort,
		artist,
		artist_sort,
		composer,
		composer_sort,
		genre,
		genre_sort,
		comments,
		comments_sort,
		grouping,
		grouping_sort,
		album_title,
		album_title_sort,
		album_artist,
		album_artist_sort,
	};
	(schema, fields)
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

fn add_opt_text(document: &mut TantivyDocument, field: Field, value: &Option<&str>) {
	if let Some(value) = value {
		document.add_text(field, value);
	}
}

fn add_opt_i64(document: &mut TantivyDocument, field: Field, value: Option<i64>) {
	if let Some(value) = value {
		document.add_i64(field, value);
	}
}

fn add_opt_f64(document: &mut TantivyDocument, field: Field, value: Option<f64>) {
	if let Some(value) = value {
		document.add_f64(field, value);
	}
}

// Builds an in-memory Tantivy index and fills it with every playlist_tracks
// row (tagged with its own playlist) plus every track (tagged with each
// "special" track list id, i.e. root).
async fn build_tantivy_index(
	paths: Paths,
	tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(Index, TantivyFields)> {
	let (schema, fields) = build_tantivy_schema();
	let tantivy_dir = Path::new(&paths.library_dir).join("Tantivy");
	std::fs::remove_dir_all(&tantivy_dir).unwrap();
	std::fs::create_dir_all(&tantivy_dir).unwrap();
	let index = Index::create_in_dir(&tantivy_dir, schema)?;
	let tokenizer = NgramTokenizer::all_ngrams(1, 3)?;
	index.tokenizers().register("subword", tokenizer);
	let mut writer: IndexWriter = index.writer(50_000_000)?;

	let start_time = Instant::now();
	let t = Instant::now();

	let all_tracks: Vec<_> = sqlx::query(
		"SELECT id, title, artist, composer, genre, comments, grouping, album_title,
			album_artist, added_at, duration_s, bpm, play_count, skip_count, year
		FROM tracks
		ORDER BY added_at ASC",
	)
	.fetch_all(&mut **tx)
	.await?;
	println!("  select all_tracks {:?}", t.elapsed());
	let t = Instant::now();

	let mut docs = Vec::with_capacity(all_tracks.len());
	for (i, row) in all_tracks.into_iter().enumerate() {
		let id: &str = row.try_get(0).unwrap();
		let title: &str = row.try_get(1).unwrap();
		let artist: &str = row.try_get(2).unwrap();
		let composer: Option<&str> = row.try_get(3).unwrap();
		let genre: Option<&str> = row.try_get(4).unwrap();
		let comments: Option<&str> = row.try_get(5).unwrap();
		let grouping: Option<&str> = row.try_get(6).unwrap();
		let album_title: Option<&str> = row.try_get(7).unwrap();
		let album_artist: Option<&str> = row.try_get(8).unwrap();
		let added_at: i64 = row.try_get(9).unwrap();
		let duration_s: f64 = row.try_get(10).unwrap();
		let bpm: Option<f64> = row.try_get(11).unwrap();
		let play_count: u64 = row.try_get(12).unwrap();
		let skip_count: u64 = row.try_get(13).unwrap();
		let year: Option<i64> = row.try_get(14).unwrap();

		let mut doc = TantivyDocument::default();
		doc.add_text(fields.track_id, &id);
		doc.add_text(fields.playlist_id, SpecialTrackListName::Root.get_id());
		doc.add_u64(fields.playlist_pos, i.try_into().unwrap());

		// Search fields
		doc.add_text(fields.title, &title);
		doc.add_text(fields.artist, &artist);
		add_opt_text(&mut doc, fields.composer, &composer);
		add_opt_text(&mut doc, fields.genre, &genre);
		add_opt_text(&mut doc, fields.comments, &comments);
		add_opt_text(&mut doc, fields.grouping, &grouping);
		add_opt_text(&mut doc, fields.album_title, &album_title);
		add_opt_text(&mut doc, fields.album_artist, &album_artist);

		// Sort fields (mirror the text values)
		doc.add_text(fields.title_sort, &title);
		doc.add_text(fields.artist_sort, &artist);
		add_opt_text(&mut doc, fields.composer_sort, &composer);
		add_opt_text(&mut doc, fields.genre_sort, &genre);
		add_opt_text(&mut doc, fields.comments_sort, &comments);
		add_opt_text(&mut doc, fields.grouping_sort, &grouping);
		add_opt_text(&mut doc, fields.album_title_sort, &album_title);
		add_opt_text(&mut doc, fields.album_artist_sort, &album_artist);

		// Numeric sorts
		doc.add_i64(fields.added_at, added_at);
		doc.add_f64(fields.duration_s, duration_s);
		add_opt_f64(&mut doc, fields.bpm, bpm);
		doc.add_u64(fields.play_count, play_count);
		doc.add_u64(fields.skip_count, skip_count);
		add_opt_i64(&mut doc, fields.year, year);

		docs.push(doc);
	}
	println!("  create docs {:?}", t.elapsed());
	let t = Instant::now();

	for doc in docs {
		writer.add_document(doc)?;
	}
	println!("  add docs {:?}", t.elapsed());
	let t = Instant::now();

	let mut playlist_tracks = sqlx::query(
		"SELECT
				playlist_tracks.track_list_id,
				playlist_tracks.item_pos,
				tracks.id,
				tracks.title,
				tracks.artist,
				tracks.composer,
				tracks.genre,
				tracks.comments,
				tracks.grouping,
				tracks.album_title,
				tracks.album_artist,
				tracks.added_at,
				tracks.duration_s,
				tracks.bpm,
				tracks.play_count,
				tracks.skip_count,
				tracks.year
		FROM playlist_tracks
		JOIN tracks ON tracks.id = playlist_tracks.track_id",
	)
	.fetch(&mut **tx);
	println!("  select playlist_tracks {:?}", t.elapsed());

	while let Some(row) = playlist_tracks.try_next().await? {
		let track_list_id: &str = row.try_get(0).unwrap();
		let item_pos: u64 = row.try_get(1).unwrap();
		let id: &str = row.try_get(2).unwrap();
		let title: &str = row.try_get(3).unwrap();
		let artist: &str = row.try_get(4).unwrap();
		let composer: Option<&str> = row.try_get(5).unwrap();
		let genre: Option<&str> = row.try_get(6).unwrap();
		let comments: Option<&str> = row.try_get(7).unwrap();
		let grouping: Option<&str> = row.try_get(8).unwrap();
		let album_title: Option<&str> = row.try_get(9).unwrap();
		let album_artist: Option<&str> = row.try_get(10).unwrap();
		let added_at: i64 = row.try_get(11).unwrap();
		let duration_s: f64 = row.try_get(12).unwrap();
		let bpm: Option<f64> = row.try_get(13).unwrap();
		let play_count: u64 = row.try_get(14).unwrap();
		let skip_count: u64 = row.try_get(15).unwrap();
		let year: Option<i64> = row.try_get(16).unwrap();

		let mut doc = TantivyDocument::default();
		doc.add_text(fields.track_id, &id);
		doc.add_text(fields.playlist_id, &track_list_id);
		doc.add_u64(fields.playlist_pos, item_pos);

		// Search fields
		doc.add_text(fields.title, &title);
		doc.add_text(fields.artist, &artist);
		add_opt_text(&mut doc, fields.composer, &composer);
		add_opt_text(&mut doc, fields.genre, &genre);
		add_opt_text(&mut doc, fields.comments, &comments);
		add_opt_text(&mut doc, fields.grouping, &grouping);
		add_opt_text(&mut doc, fields.album_title, &album_title);
		add_opt_text(&mut doc, fields.album_artist, &album_artist);

		// Sort fields (mirror the text values)
		doc.add_text(fields.title_sort, &title);
		doc.add_text(fields.artist_sort, &artist);
		add_opt_text(&mut doc, fields.composer_sort, &composer);
		add_opt_text(&mut doc, fields.genre_sort, &genre);
		add_opt_text(&mut doc, fields.comments_sort, &comments);
		add_opt_text(&mut doc, fields.grouping_sort, &grouping);
		add_opt_text(&mut doc, fields.album_title_sort, &album_title);
		add_opt_text(&mut doc, fields.album_artist_sort, &album_artist);

		// Numeric sorts
		doc.add_i64(fields.added_at, added_at);
		doc.add_f64(fields.duration_s, duration_s);
		add_opt_f64(&mut doc, fields.bpm, bpm);
		doc.add_u64(fields.play_count, play_count);
		doc.add_u64(fields.skip_count, skip_count);
		add_opt_i64(&mut doc, fields.year, year);

		writer.add_document(doc)?;
	}

	println!("  select all tracks {:?}", start_time.elapsed());

	let start = Instant::now();
	writer.commit()?;
	println!("  commit {:?}", start.elapsed());
	Ok((index, fields))
}

/// Pull track IDs from any TopDocs result regardless of its sort/score type.
fn extract_track_ids<T>(
	searcher: &Searcher,
	results: Vec<(T, DocAddress)>,
	fields: &TantivyFields,
) -> Result<Vec<String>> {
	let start_time = std::time::Instant::now();
	let mut ids = Vec::with_capacity(results.len());
	for (_, doc_address) in results {
		let doc: TantivyDocument = searcher.doc(doc_address)?;
		if let Some(id) = doc.get_first(fields.track_id).and_then(|v| v.as_str()) {
			ids.push(id.to_string());
		}
	}
	println!("extract_track_ids took {:?}", start_time.elapsed());
	Ok(ids)
}

/// Generic helper: build the collector, search, extract IDs.
fn do_search<T>(
	searcher: &Searcher,
	query: &dyn Query,
	fields: &TantivyFields,
	collector: impl Collector<Fruit = Vec<(T, DocAddress)>>,
) -> Result<Vec<String>> {
	let start_time = std::time::Instant::now();
	let results = searcher.search(query, &collector)?;
	println!("search took {:?}", start_time.elapsed());
	extract_track_ids(searcher, results, fields)
}

fn search_tantivy(
	index: &Index,
	reader: &IndexReader,
	fields: &TantivyFields,
	playlist_id: &str,
	filter_query: &str,
	sort_key: &str,
	sort_desc: bool,
) -> Result<Vec<String>> {
	let start_time = std::time::Instant::now();
	let searcher = reader.searcher();

	let playlist_term = Term::from_field_text(fields.playlist_id, playlist_id);
	let playlist_query = TermQuery::new(playlist_term, IndexRecordOption::Basic);

	let query: Box<dyn Query> = if filter_query.is_empty() {
		Box::new(playlist_query)
	} else {
		let text_fields = vec![
			fields.title,
			fields.artist,
			fields.composer,
			fields.genre,
			fields.comments,
			fields.grouping,
			fields.album_title,
			fields.album_artist,
		];
		let mut parser = QueryParser::for_index(index, text_fields);
		parser.set_conjunction_by_default();

		let text_query = parser.parse_query(filter_query)?;

		Box::new(BooleanQuery::new(vec![
			(Occur::Must, Box::new(playlist_query)),
			(Occur::Must, text_query),
		]))
	};

	let order = if sort_desc { Order::Desc } else { Order::Asc };
	let limit = searcher.num_docs().try_into().unwrap();

	let top_docs = TopDocs::with_limit(limit);
	println!("prepare search took {:?}", start_time.elapsed());
	let results = match sort_key {
		"index" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_u64_field("playlist_pos", order),
		),
		"dateAdded" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_fast_field::<i64>("added_at", order),
		),
		"duration" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_fast_field::<f64>("duration_s", order),
		),
		"bpm" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_fast_field::<f64>("bpm", order),
		),
		"playCount" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_u64_field("play_count", order),
		),
		"skipCount" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_u64_field("skip_count", order),
		),
		"year" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_fast_field::<i64>("year", order),
		),
		"name" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("title_sort", order),
		),
		"artist" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("artist_sort", order),
		),
		"albumName" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("album_title_sort", order),
		),
		"albumArtist" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("album_artist_sort", order),
		),
		"composer" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("composer_sort", order),
		),
		"genre" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("genre_sort", order),
		),
		"comments" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("comments_sort", order),
		),
		"grouping" => do_search(
			&searcher,
			&query,
			&fields,
			top_docs.order_by_string_fast_field("grouping_sort", order),
		),

		_ => panic!("Unsupported sort key {sort_key}"),
	};

	let track_ids = results?;

	Ok(track_ids)
}

#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi", napi(js_name = "get_tracks_page"))]
#[allow(dead_code)]
pub async fn get_tracks_page_js(options: TracksPageOptions) -> Result<TracksPage> {
	get_tracks_page(options).await
}

pub async fn get_tracks_page(options: TracksPageOptions) -> Result<TracksPage> {
	let mut data = Data::get_async().await;
	let paths = data.paths.clone();
	let mut tx = data.db.begin().await?;

	// todo: just for testing
	let start_time = std::time::Instant::now();
	let (tantivy_index, tantivy_fields) = build_tantivy_index(paths, &mut tx).await?;
	let tantivy_reader = tantivy_index.reader()?;
	println!("built index took {:?}", start_time.elapsed(),);

	let start_time = std::time::Instant::now();
	let track_list: TrackListPage = sqlx::query_as(
		"SELECT kind, name, description
		FROM track_lists
		WHERE id = ?",
	)
	.bind(&options.playlist_id)
	.fetch_one(&mut *tx)
	.await?;

	let filter_query = options.filter_query.trim();
	println!("select track_lists took {:?}", start_time.elapsed(),);

	let start_timex = std::time::Instant::now();
	let track_ids = search_tantivy(
		&tantivy_index,
		&tantivy_reader,
		&tantivy_fields,
		&options.playlist_id,
		filter_query,
		&options.sort_key,
		options.sort_desc,
	)?;
	println!("search_tantivity() took {:?}", start_timex.elapsed(),);

	tx.commit().await?;

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
			filter_query: "".to_string(),
			group_album_tracks: false,
		})
		.await?;

		println!("result: {:#?}", result.item_ids.len());

		Ok(())
	}
}
