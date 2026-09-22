// NOTE: This code only runs in debug mode. So you need to
// run `npm run dev` at some point before you run `npm run example-100k`

use crate::library_types::Library;
use crate::library_types::Track;
use fake::Fake;
use fake::faker::{lorem::en::Words, name::en::Name};
use std::env;

pub fn generate_test_data() {
	generate_library_100k();
}

fn words(range: std::ops::RangeInclusive<usize>) -> String {
	let range = *range.start()..(*range.end() + 1);
	let words: Vec<String> = Words(range).fake();
	words.join(" ")
}

fn generate_library_100k() {
	let path = env::current_dir()
		.unwrap()
		.join("src-native/appdata/Library100k/Library.json");
	std::fs::create_dir_all(path.parent().unwrap()).unwrap();
	let mut library = Library::new();

	for id in 0..100_000 {
		let title = words(1..=5);
		let artist = words(1..=3);
		library.try_insert_track(
			id,
			Track {
				id,
				size: (1_000_000..10_000_000).fake(),
				duration: (60.0..600.0).fake(),
				bitrate: (128.0..320.0).fake(),
				sampleRate: [44_100, 48_000, 96_000][(0..3).fake::<usize>()],
				file: format!("{artist} - {title}.mp3"),
				dateModified: (1_500_000_000_000..1_700_000_000_000).fake(),
				dateAdded: (1_500_000_000_000..1_700_000_000_000).fake(),
				name: title,
				importedFrom: None,
				originalId: None,
				artist: artist.clone(),
				composer: Name().fake(),
				sortName: None,
				sortArtist: None,
				sortComposer: None,
				genre: None,
				rating: None,
				year: Some((1980..2026).fake()),
				bpm: Some((60.0..200.0).fake()),
				comments: None,
				grouping: None,
				liked: None,
				disliked: None,
				disabled: None,
				compilation: None,
				albumName: Some(words(1..=3)),
				albumArtist: Some(artist),
				sortAlbumName: None,
				sortAlbumArtist: None,
				trackNum: None,
				trackCount: None,
				discNum: None,
				discCount: None,
				dateImported: None,
				playCount: None,
				plays: None,
				playsImported: None,
				skipCount: None,
				skips: None,
				skipsImported: None,
				volume: None,
			},
		);
	}

	std::fs::write(path, serde_json::to_vec(&library.to_file()).unwrap()).unwrap();
}
