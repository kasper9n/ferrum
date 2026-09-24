use ferrum::filter::{FilterTerm, filter};
use ferrum::library_types::ItemId;
use ferrum::migrate::LibraryFile;
use ferrum::page::TracksPageOptions;
use ferrum::sort::sort;
use ferrum::test_data::load_100k_library;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{env, fs};

fn load_100k() {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("src-native/appdata/Library100k/Library.json");

	let start = Instant::now();

	let mut data = fs::read(black_box(&path)).unwrap();
	let _: LibraryFile = simd_json::from_slice(&mut data).unwrap();

	println!("load_100k: {:?}", start.elapsed());
}

fn sort_100k() {
	let library = load_100k_library();

	let options = TracksPageOptions {
		playlist_id: "root".to_string(),
		sort_key: "name".to_string(),
		sort_desc: false,
		filter_terms: vec![],
		group_album_tracks: false,
	};

	// Warmup.
	let _ = black_box(sort(options.clone(), &library));

	let mut total = Duration::ZERO;

	let iterations: u32 = 10;
	for i in 0..10 {
		let start = Instant::now();

		let _ = black_box(sort(black_box(options.clone()), black_box(&library)));

		let elapsed = start.elapsed();
		total += elapsed;
		println!("sort_100k [{}/{iterations}]: {elapsed:?}", i + 1);
	}

	println!("sort_100k average: {:?}", total / iterations);
}

fn filter_100k() {
	let library = load_100k_library();
	let ids: Vec<ItemId> = library.get_track_item_ids().values().copied().collect();
	let terms = vec![FilterTerm {
		field: None,
		literal: "he".to_string(),
	}];

	// Warmup.
	black_box(filter(ids.clone(), terms.clone(), &library));

	let mut total = Duration::ZERO;

	let iterations: u32 = 10;
	for i in 0..iterations {
		let start = Instant::now();

		let _ = black_box(filter(ids.clone(), terms.clone(), &library));

		let elapsed = start.elapsed();
		total += elapsed;
		println!("filter_100k [{}/{iterations}]: {elapsed:?}", i + 1);
	}

	println!("filter_100k average: {:?}", total / iterations);
}

fn main() {
	let benchmark = env::args()
		.nth(1)
		.expect("specify a benchmark: load_100k, sort_100k, or filter_100k");
	match benchmark.as_str() {
		"load_100k" => load_100k(),
		"sort_100k" => sort_100k(),
		"filter_100k" => filter_100k(),
		other => panic!("unknown benchmark: {other}"),
	}
}
