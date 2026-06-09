use crate::{
	library::{Paths, load_old_library_json},
	library_types::{Library, TrackList, TrackLists},
};
use anyhow::{Context, Result};
use sqlx::{
	ConnectOptions, Connection, Sqlite, migrate::MigrateDatabase, sqlite::SqliteConnectOptions,
};
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

async fn insert_library_into_db(
	library: &Library,
	conn: &mut sqlx::SqliteConnection,
) -> anyhow::Result<()> {
	let mut tx = conn.begin().await?;

	// --- tracks ---
	for (id, track) in library.get_tracks() {
		conn.execute(
			"INSERT INTO tracks (
				id, filesize, duration_s, bitrate, sample_rate, file, modified_at, added_at, name, artist, imported_from, original_id, composer, sort_name, sort_artist, sort_composer, genre, rating_pct, year, bpm, comments, grouping, liked, disliked, disabled, compilation, album_name, album_artist, sort_album_name, sort_album_artist, track_num, track_count, disc_num, disc_count, imported_at, play_count, skip_count, volume
			) VALUES (
				:id,:filesize,:duration_s,:bitrate,:sample_rate,:file,:modified_at,:added_at,:name,:artist,:imported_from,:original_id,:composer,:sort_name,:sort_artist,:sort_composer,:genre,:rating_pct,:year,:bpm,:comments,:grouping,:liked,:disliked,:disabled,:compilation,:album_name,:album_artist,:sort_album_name,:sort_album_artist,:track_num,:track_count,:disc_num,:disc_count,:imported_at,:play_count,:skip_count,:volume
			)",
			named_params! {
				":id": track.id,
				":filesize": track.filesize,
				":duration_s": track.duration_s,
				":bitrate": track.bitrate,
				":sample_rate": track.sample_rate,
				":file": track.file,
				":modified_at": track.modified_at,
				":added_at": track.added_at,
				":name": track.name,
				":artist": track.artist,
				":imported_from": track.imported_from,
				":original_id": track.original_id,
				":composer": track.composer,
				":sort_name": track.sort_name,
				":sort_artist": track.sort_artist,
				":sort_composer": track.sort_composer,
				":genre": track.genre,
				":rating_pct": track.rating_pct,
				":year": track.year,
				":bpm": track.bpm,
				":comments": track.comments,
				":grouping": track.grouping,
				":liked": track.liked,
				":disliked": track.disliked,
				":disabled": track.disabled,
				":compilation": track.compilation,
				":album_name": track.album_name,
				":album_artist": track.album_artist,
				":sort_album_name": track.sort_album_name,
				":sort_album_artist": track.sort_album_artist,
				":track_num": track.track_num,
				":track_count": track.track_count,
				":disc_num": track.disc_num,
				":disc_count": track.disc_count,
				":imported_at": track.imported_at,
				":play_count": track.play_count,
				":skip_count": track.skip_count,
				":volume": track.volume,

			},
		)?;
		sqlx::query!(
			r#"
				INSERT INTO tracks (
				id
				filesize
				duration_s
				bitrate
				sample_rate
				file
				modified_at
				added_at
				name
				artist
				imported_from
				original_id
				composer
				sort_name
				sort_artist
				sort_composer
				genre
				rating_pct
				year
				bpm
				comments
				grouping
				liked
				disliked
				disabled
				compilation
				album_name
				album_artist
				sort_album_name
				sort_album_artist
				track_num
				track_count
				disc_num
				disc_count
				imported_at
				play_count
				skip_count
				volume
					) VALUES (
						?1, ?2, ?3, ?4, ?5, ?6,
						?7, ?8, ?9, ?10,
						?11, ?12, ?13,
						?14, ?15, ?16,
						?17, ?18, ?19, ?20, ?21, ?22,
						?23, ?24, ?25, ?26,
						?27, ?28, ?29, ?30,
						?31, ?32, ?33, ?34,
						?35, ?36, ?37, ?38
		      )
				"#,
			id,
			track.size,
			track.duration,
			track.bitrate,
			track.sampleRate,
			track.file,
			track.dateModified,
			track.dateAdded,
			track.name,
			track.artist,
			track.importedFrom,
			track.originalId,
			track.composer,
			track.sortName,
			track.sortArtist,
			track.sortComposer,
			track.genre,
			track.rating,
			track.year,
			track.bpm,
			track.comments,
			track.grouping,
			track.liked,
			track.disliked,
			track.disabled,
			track.compilation,
			track.albumName,
			track.albumArtist,
			track.sortAlbumName,
			track.sortAlbumArtist,
			track.trackNum,
			track.trackCount,
			track.discNum,
			track.discCount,
			track.dateImported,
			track.playCount,
			track.skipCount,
			track.volume,
		)
		.execute(&mut *tx)
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
