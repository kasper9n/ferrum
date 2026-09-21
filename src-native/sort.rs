use crate::library::get_tracklist_item_ids;
use crate::library_types::{ItemId, Library, TRACK_ID_MAP, Track};
use crate::page::TracksPageOptions;
use alphanumeric_sort::compare_str;
use anyhow::{Context, Result};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use std::cmp::Ordering;
use std::time::Instant;

pub type TracksPageOptionsX = TracksPageOptions;

struct SortItem<'a> {
	item_id: ItemId,
	track: &'a Track,
}

pub fn sort(options: TracksPageOptions, library: &Library) -> Result<Vec<ItemId>> {
	let now = Instant::now();

	let id_map = TRACK_ID_MAP.read().unwrap();
	let tracks = library.get_tracks();

	let items: Result<Vec<SortItem>> = get_tracklist_item_ids(library, &options.playlist_id)?
		.into_par_iter()
		.enumerate()
		.map(|(i, id)| {
			Ok(SortItem {
				item_id: id,
				track: tracks.get(&id_map[id as usize]).context(format!(
					"Track {i} ({}) does not exist",
					id_map[id as usize]
				))?,
			})
		})
		.collect();
	let mut items = items?;
	let item_count = items.len();

	if options.sort_key == "index" {
		// Note: Indexes descend from "first to last", unlike
		// other numbers which ascend from "high to low"
		if !options.sort_desc {
			items.reverse();
		}
		println!("Sort: {}ms", now.elapsed().as_millis());
		let item_ids = items.into_iter().map(|item| item.item_id).collect();
		return Ok(item_ids);
	}

	let group_album_tracks = options.group_album_tracks
		&& match options.sort_key.as_str() {
			"dateAdded" | "albumName" | "comments" | "genre" | "year" | "artist" => true,
			_ => false,
		};

	match options.sort_key.as_str() {
		"file" => items.par_sort_by(|a, b| cmp_str(&a.track.file, &b.track.file)),
		"name" => items.par_sort_by(|a, b| cmp_str(&a.track.name, &b.track.name)),
		"artist" => items.par_sort_by(|a, b| cmp_str(&a.track.artist, &b.track.artist)),
		"importedFrom" => items.par_sort_by(|a, b| {
			cmp_opt_str(
				a.track.importedFrom.as_deref(),
				b.track.importedFrom.as_deref(),
			)
		}),
		"originalId" => items.par_sort_by(|a, b| {
			cmp_opt_str(a.track.originalId.as_deref(), b.track.originalId.as_deref())
		}),
		"composer" => items.par_sort_by(|a, b| {
			cmp_opt_str(a.track.composer.as_deref(), b.track.composer.as_deref())
		}),
		"sortName" => items.par_sort_by(|a, b| {
			cmp_opt_str(a.track.sortName.as_deref(), b.track.sortName.as_deref())
		}),
		"sortArtist" => items.par_sort_by(|a, b| {
			cmp_opt_str(a.track.sortArtist.as_deref(), b.track.sortArtist.as_deref())
		}),
		"sortComposer" => items.par_sort_by(|a, b| {
			cmp_opt_str(
				a.track.sortComposer.as_deref(),
				b.track.sortComposer.as_deref(),
			)
		}),
		"genre" => items
			.par_sort_by(|a, b| cmp_opt_str(a.track.genre.as_deref(), b.track.genre.as_deref())),
		"comments" => items.par_sort_by(|a, b| {
			cmp_opt_str(a.track.comments.as_deref(), b.track.comments.as_deref())
		}),
		"grouping" => items.par_sort_by(|a, b| {
			cmp_opt_str(a.track.grouping.as_deref(), b.track.grouping.as_deref())
		}),
		"albumName" => items.par_sort_by(|a, b| {
			cmp_opt_str(a.track.albumName.as_deref(), b.track.albumName.as_deref())
		}),
		"albumArtist" => items.par_sort_by(|a, b| {
			cmp_opt_str(
				a.track.albumArtist.as_deref(),
				b.track.albumArtist.as_deref(),
			)
		}),
		"sortAlbumName" => items.par_sort_by(|a, b| {
			cmp_opt_str(
				a.track.sortAlbumName.as_deref(),
				b.track.sortAlbumName.as_deref(),
			)
		}),
		"sortAlbumArtist" => items.par_sort_by(|a, b| {
			cmp_opt_str(
				a.track.sortAlbumArtist.as_deref(),
				b.track.sortAlbumArtist.as_deref(),
			)
		}),
		"duration" => items.par_sort_by(|a, b| cmp_f64(a.track.duration, b.track.duration)),
		"bitrate" => items.par_sort_by(|a, b| cmp_f64(a.track.bitrate, b.track.bitrate)),
		"sampleRate" => items.par_sort_by(|a, b| cmp_f64(a.track.sampleRate, b.track.sampleRate)),
		"bpm" => items
			.par_sort_by(|a, b| cmp_f64(a.track.bpm.unwrap_or(0.0), b.track.bpm.unwrap_or(0.0))),
		"size" => items.par_sort_by(|a, b| a.track.size.cmp(&b.track.size)),
		"dateModified" => items.par_sort_by(|a, b| a.track.dateModified.cmp(&b.track.dateModified)),
		"dateAdded" => items.par_sort_by(|a, b| a.track.dateAdded.cmp(&b.track.dateAdded)),
		"dateImported" => items.par_sort_by(|a, b| {
			a.track
				.dateImported
				.unwrap_or(0)
				.cmp(&b.track.dateImported.unwrap_or(0))
		}),
		"year" => {
			items.par_sort_by(|a, b| a.track.year.unwrap_or(0).cmp(&b.track.year.unwrap_or(0)))
		}
		"trackNum" => items.par_sort_by(|a, b| {
			a.track
				.trackNum
				.unwrap_or(0)
				.cmp(&b.track.trackNum.unwrap_or(0))
		}),
		"trackCount" => items.par_sort_by(|a, b| {
			a.track
				.trackCount
				.unwrap_or(0)
				.cmp(&b.track.trackCount.unwrap_or(0))
		}),
		"discNum" => items.par_sort_by(|a, b| {
			a.track
				.discNum
				.unwrap_or(0)
				.cmp(&b.track.discNum.unwrap_or(0))
		}),
		"discCount" => items.par_sort_by(|a, b| {
			a.track
				.discCount
				.unwrap_or(0)
				.cmp(&b.track.discCount.unwrap_or(0))
		}),
		"playCount" => items.par_sort_by(|a, b| {
			a.track
				.playCount
				.unwrap_or(0)
				.cmp(&b.track.playCount.unwrap_or(0))
		}),
		"skipCount" => items.par_sort_by(|a, b| {
			a.track
				.skipCount
				.unwrap_or(0)
				.cmp(&b.track.skipCount.unwrap_or(0))
		}),
		"volume" => items.par_sort_by(|a, b| {
			a.track
				.volume
				.unwrap_or(0)
				.cmp(&b.track.volume.unwrap_or(0))
		}),
		"rating" => items.par_sort_by(|a, b| {
			a.track
				.rating
				.unwrap_or(0)
				.cmp(&b.track.rating.unwrap_or(0))
		}),
		"liked" => items.par_sort_by(|a, b| {
			a.track
				.liked
				.unwrap_or(false)
				.cmp(&b.track.liked.unwrap_or(false))
		}),
		"disliked" => items.par_sort_by(|a, b| {
			a.track
				.disliked
				.unwrap_or(false)
				.cmp(&b.track.disliked.unwrap_or(false))
		}),
		"disabled" => items.par_sort_by(|a, b| {
			a.track
				.disabled
				.unwrap_or(false)
				.cmp(&b.track.disabled.unwrap_or(false))
		}),
		"compilation" => items.par_sort_by(|a, b| {
			a.track
				.compilation
				.unwrap_or(false)
				.cmp(&b.track.compilation.unwrap_or(false))
		}),
		_ => panic!("Unknown sort key {}", options.sort_key),
	}

	if options.sort_desc {
		items.reverse();
	}

	if group_album_tracks {
		let mut post_grouped_items: Vec<_> = Vec::with_capacity(items.len());
		let mut items_iter = items.into_iter().peekable();

		// Process the first track in the next album
		while let Some(first_item) = items_iter.next() {
			let first_track = first_item.track;
			// We need to get the first track to compare with the later tracks
			let mut current_album_buffer: Vec<SortItem> = vec![first_item];

			// Collect the rest of the tracks from the same album
			while let Some(item) = items_iter.peek() {
				if !item.track.is_same_album(first_track) {
					break;
				}
				let item = items_iter.next().unwrap();
				current_album_buffer.push(item);
			}

			// Sort album tracks by discNum, then trackNum
			current_album_buffer.sort_by(|a, b| {
				let order = a
					.track
					.discNum
					.unwrap_or(0)
					.cmp(&b.track.discNum.unwrap_or(0));
				if order == Ordering::Equal {
					a.track
						.trackNum
						.unwrap_or(0)
						.cmp(&b.track.trackNum.unwrap_or(0))
				} else {
					order
				}
			});

			post_grouped_items.append(&mut current_album_buffer);
		}

		assert_eq!(item_count, post_grouped_items.len());
		items = post_grouped_items;
	}

	println!("Sort: {}ms", now.elapsed().as_millis());
	let item_ids = items.into_iter().map(|item| item.item_id).collect();
	return Ok(item_ids);
}

#[inline]
fn cmp_str(a: &str, b: &str) -> Ordering {
	if a == "" && b == "" {
		return Ordering::Equal;
	}
	if a == "" {
		return Ordering::Greater;
	}
	if b == "" {
		return Ordering::Less;
	}
	return compare_str(a, b);
}

#[inline]
fn cmp_opt_str(a: Option<&str>, b: Option<&str>) -> Ordering {
	cmp_str(a.unwrap_or(""), b.unwrap_or(""))
}

#[inline]
fn cmp_f64(a: f64, b: f64) -> Ordering {
	a.partial_cmp(&b)
		.unwrap_or_else(|| panic!("Unable to compare f64 {} and {}", a, b))
}
