use crate::{
	library::{Paths, load_old_library_json},
	library_types::{Library, MsSinceUnixEpoch, PercentInteger},
};
use anyhow::{Context, Result};
use serde::Serialize;
use sqlx::{
	ConnectOptions, Connection, Executor, QueryBuilder, Sqlite, migrate::MigrateDatabase,
	sqlite::SqliteConnectOptions,
};
use struct_field_names_as_array::FieldNamesAsArray;
use tempfile::TempDir;

pub async fn migrate_to_sqlite(paths: &Paths) -> Result<()> {
	let old_library = match load_old_library_json(&paths.library_json)? {
		None => {
			return Ok(());
		}
		Some(old_library) => old_library,
	};

	let tmp_dir = TempDir::new().context("failed to create temp dir")?;
	let tmp_db = tmp_dir.path().join("Library.sqlite");

	Sqlite::create_database(&tmp_db.to_str().unwrap())
		.await
		.context("Could not create library database")?;
	let mut connection = SqliteConnectOptions::new()
		.filename(&tmp_db)
		.connect()
		.await
		.context("Error connecting to created library database")?;

	sqlx::migrate!("./src-native/migrations")
		.run(&mut connection)
		.await
		.context("Could not run database migrations")?;

	insert_library_into_db(&old_library, &mut connection)
		.await
		.context("could not insert library into database")?;

	connection
		.close()
		.await
		.context("Could not save/close database")?;

	std::fs::rename(&tmp_db, &paths.library_sqlite)
		.context("Failed to finalize sqlite database")?;

	Ok(())
}

#[derive(FieldNamesAsArray, Serialize)]
pub struct Track {
	pub id: i64,
	pub filesize: i64,
	pub duration_s: f64,
	pub bitrate: f64,
	pub sample_rate: f64,
	pub file: String,
	pub modified_at: MsSinceUnixEpoch,
	pub added_at: MsSinceUnixEpoch,
	pub name: String,
	pub imported_from: Option<String>,
	pub original_id: Option<String>, // Imported ID, like iTunes Persistent ID
	pub artist: String,
	pub composer: Option<String>,
	pub sort_name: Option<String>,
	pub sort_artist: Option<String>,
	pub sort_composer: Option<String>,
	pub genre: Option<String>,
	pub rating_pct: Option<PercentInteger>,
	pub year: Option<i64>,
	pub bpm: Option<f64>,
	pub comments: Option<String>,
	pub grouping: Option<String>,
	pub liked: Option<bool>,
	pub disliked: Option<bool>,
	pub disabled: Option<bool>,
	pub compilation: Option<bool>,
	pub album_name: Option<String>,
	pub album_artist: Option<String>,
	pub sort_album_name: Option<String>,
	pub sort_album_artist: Option<String>,
	pub track_num: Option<u32>,
	pub track_count: Option<u32>,
	pub disc_num: Option<u32>,
	pub disc_count: Option<u32>,
	pub imported_at: Option<MsSinceUnixEpoch>,
	pub play_count: Option<u32>,
	pub skip_count: Option<u32>,
	pub volume: Option<i8>, // -100 to 100
}

#[macro_export]
macro_rules! insert_sql {
    (
        $qb:expr,
        $table:expr,
        $struct_ty:ty,
        $(
            $col:ident : $val:expr
        ),+ $(,)?
    ) => {{
        // Compile-time-ish validation block.
        {
            let expected = <$struct_ty>::FIELD_NAMES_AS_ARRAY;
            let provided = [
                $(
                    stringify!($col),
                )+
            ];

            assert!(
                expected.len() == provided.len(),
                "Expected {} fields for {}, got {}",
                expected.len(),
                stringify!($struct_ty),
                provided.len(),
            );

            for field in expected {
                assert!(
                    provided.contains(&field),
                    "Missing field `{}` for {}",
                    field,
                    stringify!($struct_ty),
                );
            }
        }

        $qb.push("INSERT INTO ");
        $qb.push("\"");
        $qb.push($table);
        $qb.push("\"");

        $qb.push(" (");
        {
            let mut separated = $qb.separated(", ");
            $(
                separated.push(stringify!($col));
            )+
        }

        $qb.push(") VALUES (");

        {
            let mut separated = $qb.separated(", ");
            $(
                separated.push_bind($val.clone());
            )+
        }

        $qb.push(")");
    }};
}

async fn insert_library_into_db(
	library: &Library,
	conn: &mut sqlx::SqliteConnection,
) -> anyhow::Result<()> {
	let mut tx = conn.begin().await?;

	// --- tracks ---
	for (id, track) in library.get_tracks() {
		let mut query = QueryBuilder::<Sqlite>::new("");
		insert_sql!(
			query,
			"tracks",
			Track,
			id: id,
			filesize: track.size,
			duration_s: track.duration,
			bitrate: track.bitrate,
			sample_rate: track.sampleRate,
			file: track.file,
			modified_at: track.dateModified,
			added_at: track.dateAdded,
			name: track.name,
			imported_from: track.importedFrom,
			original_id: track.originalId,
			artist: track.artist,
			composer: track.composer,
			sort_name: track.sortName,
			sort_artist: track.sortArtist,
			sort_composer: track.sortComposer,
			genre: track.genre,
			rating_pct: track.rating,
			year: track.year,
			bpm: track.bpm,
			comments: track.comments,
			grouping: track.grouping,
			liked: track.liked,
			disliked: track.disliked,
			disabled: track.disabled,
			compilation: track.compilation,
			album_name: track.albumName,
			album_artist: track.albumArtist,
			sort_album_name: track.sortAlbumName,
			sort_album_artist: track.sortAlbumArtist,
			track_num: track.trackNum,
			track_count: track.trackCount,
			disc_num: track.discNum,
			disc_count: track.discCount,
			imported_at: track.dateImported,
			play_count: track.playCount,
			skip_count: track.skipCount,
			volume: track.volume,
		);
		tx.execute(query.build())
			.await
			.with_context(|| format!("Failed to insert track {id}"))?;

		// 	// Individual play timestamps (known exact times)
		// 	if let Some(plays) = &track.plays {
		// 		for &date in plays {
		// 			sqlx::query!(
		// 				"INSERT INTO plays (track_id, date, date_range_to) VALUES (?1, ?2, NULL)",
		// 				id,
		// 				date,
		// 			)
		// 			.execute(&mut *tx)
		// 			.await?;
		// 		}
		// 	}

		// 	// Imported play ranges (CountObject = fromDate..toDate with a count,
		// 	// but the schema only stores one row per range — store as a range row
		// 	// and ignore count since the schema has no count column)
		// 	if let Some(imported) = &track.playsImported {
		// 		for co in imported {
		// 			sqlx::query!(
		// 				"INSERT INTO plays (track_id, date, date_range_to) VALUES (?1, ?2, ?3)",
		// 				id,
		// 				co.fromDate,
		// 				co.toDate,
		// 			)
		// 			.execute(&mut *tx)
		// 			.await?;
		// 		}
		// 	}

		// 	// Individual skip timestamps
		// 	if let Some(skips) = &track.skips {
		// 		for &date in skips {
		// 			sqlx::query!(
		// 				"INSERT INTO skips (track_id, date, date_range_to) VALUES (?1, ?2, NULL)",
		// 				id,
		// 				date,
		// 			)
		// 			.execute(&mut *tx)
		// 			.await?;
		// 		}
		// 	}

		// 	// Imported skip ranges
		// 	if let Some(imported) = &track.skipsImported {
		// 		for co in imported {
		// 			sqlx::query!(
		// 				"INSERT INTO skips (track_id, date, date_range_to) VALUES (?1, ?2, ?3)",
		// 				id,
		// 				co.fromDate,
		// 				co.toDate,
		// 			)
		// 			.execute(&mut *tx)
		// 			.await?;
		// 		}
		// 	}
		// }

		// // --- track_lists (folders, playlists, specials) ---
		// // We need parent_id, which requires a first pass to build the parent map.
		// let parent_map = build_parent_map(&library.trackLists);

		// for (list_id, tracklist) in &library.trackLists {
		// 	let parent_id = parent_map.get(list_id.as_str()).map(|s| s.as_str());
		// 	let position = get_position_in_parent(list_id, &library.trackLists);

		// 	match tracklist {
		// 		TrackList::Special(s) => {
		// 			let name = s.name.to_string();
		// 			sqlx::query!(
		// 				r#"INSERT INTO track_lists
		//                        (id, type, parent_id, position, name, description, liked, disliked,
		//                         imported_from, original_id, imported_at, created_at)
		//                       VALUES (?1,'special',?2,?3,?4,'',0,0,NULL,NULL,NULL,?5)"#,
		// 				s.id,
		// 				parent_id,
		// 				position,
		// 				name,
		// 				s.dateCreated,
		// 			)
		// 			.execute(&mut *tx)
		// 			.await?;
		// 		}
		// 		TrackList::Folder(f) => {
		// 			sqlx::query!(
		// 				r#"INSERT INTO track_lists
		//                        (id, type, parent_id, position, name, description, liked, disliked,
		//                         imported_from, original_id, imported_at, created_at)
		//                       VALUES (?1,'folder',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"#,
		// 				f.id,
		// 				parent_id,
		// 				position,
		// 				f.name,
		// 				f.description,
		// 				f.liked,
		// 				f.disliked,
		// 				f.importedFrom,
		// 				f.originalId,
		// 				f.dateImported,
		// 				f.dateCreated,
		// 			)
		// 			.execute(&mut *tx)
		// 			.await?;
		// 		}
		// 		TrackList::Playlist(p) => {
		// 			sqlx::query!(
		// 				r#"INSERT INTO track_lists
		//                        (id, type, parent_id, position, name, description, liked, disliked,
		//                         imported_from, original_id, imported_at, created_at)
		//                       VALUES (?1,'playlist',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)"#,
		// 				p.id,
		// 				parent_id,
		// 				position,
		// 				p.name,
		// 				p.description,
		// 				p.liked,
		// 				p.disliked,
		// 				p.importedFrom,
		// 				p.originalId,
		// 				p.dateImported,
		// 				p.dateCreated,
		// 			)
		// 			.execute(&mut *tx)
		// 			.await?;

		// 			// playlist_tracks rows
		// 			for (pos, track_id) in p.get_track_ids().iter().enumerate() {
		// 				let pos = pos as i64;
		// 				sqlx::query!(
		// 					"INSERT INTO playlist_tracks (track_list_id, track_id, position)
		//                         VALUES (?1, ?2, ?3)",
		// 					p.id,
		// 					track_id,
		// 					pos,
		// 				)
		// 				.execute(&mut *tx)
		// 				.await?;
		// 			}
		// 		}
		// 	}
	}

	// // --- play_times (v1 and v2) ---
	// for (track_id, started_at, duration) in &library.v1PlayTime {
	// 	sqlx::query!(
	// 		"INSERT INTO play_times (track_id, started_at, duration, is_v1) VALUES (?1,?2,?3,1)",
	// 		track_id,
	// 		started_at,
	// 		duration,
	// 	)
	// 	.execute(&mut *tx)
	// 	.await?;
	// }
	// for (track_id, started_at, duration) in &library.playTime {
	// 	sqlx::query!(
	// 		"INSERT INTO play_times (track_id, started_at, duration, is_v1) VALUES (?1,?2,?3,0)",
	// 		track_id,
	// 		started_at,
	// 		duration,
	// 	)
	// 	.execute(&mut *tx)
	// 	.await?;
	// }

	tx.commit()
		.await
		.context("Failed to commit migration transaction")?;
	Ok(())
}

// /// Returns a map of child_id -> parent_id for every tracklist entry.
// fn build_parent_map(track_lists: &TrackLists) -> std::collections::HashMap<String, String> {
// 	let mut map = std::collections::HashMap::new();
// 	for (parent_id, tl) in track_lists {
// 		let children = match tl {
// 			TrackList::Folder(f) => &f.children,
// 			TrackList::Special(s) => &s.children,
// 			TrackList::Playlist(_) => continue,
// 		};
// 		for child_id in children {
// 			map.insert(child_id.clone(), parent_id.clone());
// 		}
// 	}
// 	map
// }

// /// Returns the 0-based position of `id` in its parent's children list, or None.
// fn get_position_in_parent(id: &str, track_lists: &TrackLists) -> Option<i64> {
// 	for tl in track_lists.values() {
// 		let children = match tl {
// 			TrackList::Folder(f) => &f.children,
// 			TrackList::Special(s) => &s.children,
// 			TrackList::Playlist(_) => continue,
// 		};
// 		if let Some(pos) = children.iter().position(|c| c == id) {
// 			return Some(pos as i64);
// 		}
// 	}
// 	None
// }
