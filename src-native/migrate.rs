use crate::{
	library::{Paths, load_old_library_json},
	library_types::{Library, TrackList, TrackLists},
};
use anyhow::{Context, Result};
use sqlx::{
	ConnectOptions, Connection, Sqlite, migrate::MigrateDatabase, sqlite::SqliteConnectOptions,
};
use std::time::Instant;
use tempfile::TempDir;

pub async fn migrate_to_sqlite(paths: &Paths) -> Result<()> {
	let now = Instant::now();

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
		.run_to(1, &mut connection)
		.await
		.context("Could not run database migrations")?;

	insert_library_into_db(&old_library, &mut connection)
		.await
		.context("Could not insert Library.json into database")?;

	connection
		.close()
		.await
		.context("Could not save/close database")?;

	std::fs::rename(&tmp_db, &paths.library_sqlite)
		.context("Failed to finalize sqlite database")?;

	println!("Migrated to SQLite: {}ms", now.elapsed().as_millis());

	Ok(())
}

async fn insert_library_into_db(
	library: &Library,
	conn: &mut sqlx::SqliteConnection,
) -> anyhow::Result<()> {
	let mut tx = conn.begin().await.context("Failed to begin transaction")?;

	for (track_id, track) in library.get_tracks() {
		sqlx::query(
			"
				INSERT INTO tracks (
					id,
					filesize,
					duration_s,
					bitrate,
					sample_rate,
					file,
					modified_at,
					added_at,
					name,
					artist,
					imported_from,
					original_id,
					composer,
					sort_name,
					sort_artist,
					sort_composer,
					genre,
					rating_pct,
					year,
					bpm,
					comments,
					grouping,
					liked,
					disliked,
					disabled,
					compilation,
					album_name,
					album_artist,
					sort_album_name,
					sort_album_artist,
					track_num,
					track_count,
					disc_num,
					disc_count,
					imported_at,
					play_count,
					skip_count,
					volume
				) VALUES (
					?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
					?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
					?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
					?, ?, ?, ?, ?, ?, ?, ?
				)
			",
		)
		.bind(track_id)
		.bind(track.size)
		.bind(track.duration)
		.bind(track.bitrate)
		.bind(track.sampleRate)
		.bind(&track.file)
		.bind(track.dateModified)
		.bind(track.dateAdded)
		.bind(&track.name)
		.bind(&track.artist)
		.bind(&track.importedFrom)
		.bind(&track.originalId)
		.bind(&track.composer)
		.bind(&track.sortName)
		.bind(&track.sortArtist)
		.bind(&track.sortComposer)
		.bind(&track.genre)
		.bind(track.rating)
		.bind(track.year)
		.bind(track.bpm)
		.bind(&track.comments)
		.bind(&track.grouping)
		.bind(track.liked)
		.bind(track.disliked)
		.bind(track.disabled)
		.bind(track.compilation)
		.bind(&track.albumName)
		.bind(&track.albumArtist)
		.bind(&track.sortAlbumName)
		.bind(&track.sortAlbumArtist)
		.bind(track.trackNum)
		.bind(track.trackCount)
		.bind(track.discNum)
		.bind(track.discCount)
		.bind(track.dateImported)
		.bind(track.playCount)
		.bind(track.skipCount)
		.bind(track.volume)
		.execute(&mut *tx)
		.await
		.with_context(|| format!("Failed to insert track {track_id}"))?;

		if let Some(plays) = &track.plays {
			for &date in plays {
				sqlx::query("INSERT INTO plays (date, track_id) VALUES (?, ?)")
					.bind(date)
					.bind(track_id)
					.execute(&mut *tx)
					.await
					.with_context(|| format!("Failed to insert plays"))?;
			}
		}

		if let Some(imported) = &track.playsImported {
			for co in imported {
				sqlx::query(
					"INSERT INTO plays_imported (date_range_from, date_range_to, count, track_id) VALUES (?, ?, ?, ?)",
				)
				.bind(co.fromDate)
				.bind(co.toDate)
				.bind(co.count)
				.bind(track_id)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert plays_imported"))?;
			}
		}

		if let Some(skips) = &track.skips {
			for &date in skips {
				sqlx::query("INSERT INTO skips (date, track_id) VALUES (?, ?)")
					.bind(date)
					.bind(track_id)
					.execute(&mut *tx)
					.await
					.with_context(|| format!("Failed to insert skips"))?;
			}
		}

		if let Some(imported) = &track.skipsImported {
			for co in imported {
				sqlx::query(
					"INSERT INTO skips_imported (date_range_from, date_range_to, count, track_id) VALUES (?, ?, ?, ?)",
				)
				.bind(co.fromDate)
				.bind(co.toDate)
				.bind(co.count)
				.bind(track_id)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert skips_imported"))?;
			}
		}
	}

	for (track_id, started_at, duration) in &library.v1PlayTime {
		sqlx::query(
			"INSERT INTO play_times (started_at, duration, track_id, is_v1) VALUES (?, ?, ?, 1)",
		)
		.bind(started_at)
		.bind(duration)
		.bind(track_id)
		.execute(&mut *tx)
		.await
		.with_context(|| format!("Failed to insert v1 play_times"))?;
	}
	for (track_id, started_at, duration) in &library.playTime {
		sqlx::query(
			"INSERT INTO play_times (started_at, duration, track_id, is_v1) VALUES (?, ?, ?, 1)",
		)
		.bind(started_at)
		.bind(duration)
		.bind(track_id)
		.execute(&mut *tx)
		.await
		.with_context(|| format!("Failed to insert play_times"))?;
	}

	let parent_map = build_parent_map(&library.trackLists);

	for (list_id, tracklist) in &library.trackLists {
		match tracklist {
			TrackList::Special(special) => {
				let name = special.name.to_string();
				sqlx::query(
					"
						INSERT INTO track_lists
							(id, type, name, description, created_at)
						VALUES (?, ?, ?, ?, ?)
					",
				)
				.bind(&special.id)
				.bind("folder")
				.bind(special.name.to_string())
				.bind("")
				.bind(special.dateCreated)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert special playlist {name}"))?;
			}
			TrackList::Folder(folder) => {
				let (index, parent_id) = parent_map
					.get(list_id.as_str())
					.with_context(|| format!("Parent of folder {} not found", folder.name))?;
				sqlx::query(
					"
						INSERT INTO track_lists
							(id, type, parent_id, item_index, name, description, liked, disliked,
							imported_from, original_id, imported_at, created_at)
						VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					",
				)
				.bind(&folder.id)
				.bind("folder")
				.bind(parent_id)
				.bind(index)
				.bind(&folder.name)
				.bind(&folder.description)
				.bind(folder.liked)
				.bind(folder.disliked)
				.bind(&folder.importedFrom)
				.bind(&folder.originalId)
				.bind(folder.dateImported)
				.bind(folder.dateCreated)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert playlist folder {}", folder.name))?;
			}
			TrackList::Playlist(playlist) => {
				let (index, parent_id) = parent_map
					.get(list_id.as_str())
					.with_context(|| format!("Parent of playlist {} not found", playlist.name))?;
				sqlx::query(
					"
						INSERT INTO track_lists
							(id, type, parent_id, item_index, name, description, liked, disliked,
							imported_from, original_id, imported_at, created_at)
						VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					",
				)
				.bind(&playlist.id)
				.bind("folder")
				.bind(parent_id)
				.bind(index)
				.bind(&playlist.name)
				.bind(&playlist.description)
				.bind(playlist.liked)
				.bind(playlist.disliked)
				.bind(&playlist.importedFrom)
				.bind(&playlist.originalId)
				.bind(playlist.dateImported)
				.bind(playlist.dateCreated)
				.execute(&mut *tx)
				.await
				.with_context(|| format!("Failed to insert playlist {}", playlist.name))?;

				// playlist_tracks rows
				for (i, track_id) in playlist.get_track_ids().iter().enumerate() {
					let i: i64 = i.try_into().unwrap();
					assert!(i >= 0);
					sqlx::query(
						"INSERT INTO playlist_tracks (track_list_id, track_id, item_index) VALUES (?, ?, ?)",
					)
					.bind(&playlist.id)
					.bind(track_id)
					.bind(i)
					.execute(&mut *tx)
					.await
					.with_context(|| {
						format!(
							"Failed to insert track {} in playlist {}",
							track_id, playlist.name
						)
					})?;
				}
			}
		}
	}

	tx.commit().await.context("Failed to commit transaction")?;
	Ok(())
}

/// Returns a map of playlist -> (index, parent_id) for every tracklist entry.
fn build_parent_map(track_lists: &TrackLists) -> std::collections::HashMap<String, (i64, String)> {
	let mut map = std::collections::HashMap::new();
	for (parent_id, tl) in track_lists {
		let children = match tl {
			TrackList::Folder(f) => &f.children,
			TrackList::Special(s) => &s.children,
			TrackList::Playlist(_) => continue,
		};
		for (i, child_id) in children.iter().enumerate() {
			// SQLite does not support u64
			let i: i64 = i.try_into().unwrap();
			assert!(i >= 0);
			map.insert(child_id.clone(), (i, parent_id.clone()));
		}
	}
	map
}
