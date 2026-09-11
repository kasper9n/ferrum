use anyhow::{Context, Result, bail};
use ferrum::library::Paths;
use ferrum::library_types::{Library, TRACK_ID_MAP, Track, TrackList, TrackListID};
use ferrum::migrate::{self, LibraryFile};
use ferrum::page::{TracksPage, TracksPageOptions, get_tracks_page_from_library};
use ferrum::path_to_string;
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_specta::Builder;

#[tauri::command]
#[specta::specta]
fn error_popup(app_handle: tauri::AppHandle, msg: String) {
	eprintln!("Error: {}", msg);

	app_handle
		.dialog()
		.message(&msg)
		.kind(MessageDialogKind::Error)
		.title("Error")
		.show(|_| {});
}

#[tauri::command]
#[specta::specta]
async fn open_file_persistent_android(app: AppHandle) -> Result<Option<String>, String> {
	if cfg!(not(target_os = "android")) {
		panic!("Persistent save dialog cannot be called on this platform");
	}

	use tauri_plugin_android_fs::AndroidFsExt;
	let api = app.android_fs_async();

	let selected_file = api
		.file_picker()
		.pick_file(
			None,                  // Initial location
			&["application/json"], // Target MIME types
			false,                 // If true, only files on local device
		)
		.await;
	let selected_file = match selected_file {
		Ok(Some(file)) => file,
		Ok(None) => return Ok(None),
		Err(err) => return Err(format!("Error picking file: {err}")),
	};

	// Persist file access between restarts
	match api
		.file_picker()
		.persist_uri_permission(&selected_file)
		.await
	{
		Ok(_) => {}
		Err(err) => return Err(format!("Error persisting URI: {err}")),
	};

	Ok(Some(selected_file.uri.to_string()))
}

fn load_library_from_file(library_json: &str, paths: &Paths) -> anyhow::Result<Library> {
	let now = Instant::now();

	let mut json_bytes = library_json.as_bytes().to_vec();

	let versioned_library: LibraryFile = match simd_json::from_slice(&mut json_bytes) {
		Ok(lib) => {
			println!("Parsed library: {}ms", now.elapsed().as_millis());
			lib
		}
		Err(err) => {
			bail!("Error parsing library: {}", err);
		}
	};
	let now = Instant::now();

	let latest_library = migrate::upgrade(versioned_library, paths)?;
	let library = Library::init_library(latest_library);
	println!("Initialized library: {}ms", now.elapsed().as_millis());
	Ok(library)
}

#[tauri::command]
#[specta::specta]
fn get_track_by_item_id(item_id: u32, app: AppHandle) -> Result<Track, String> {
	let library_state = app.state::<Mutex<Library>>();
	let library = library_state.lock().unwrap();
	let id_map = TRACK_ID_MAP.read().unwrap();
	let track_id = &id_map[item_id as usize];
	let track = library.get_track(&track_id).expect("Could not get track");
	Ok(track.clone())
}

#[tauri::command]
#[specta::specta]
fn get_tracks_page(options: TracksPageOptions, app: AppHandle) -> Result<TracksPage, String> {
	let library_state = app.state::<Mutex<Library>>();
	let library = library_state.lock().unwrap();
	match get_tracks_page_from_library(options, &library) {
		Ok(page) => Ok(page),
		Err(err) => Err(err.to_string()),
	}
}

#[derive(Serialize, Type)]
pub struct LibraryTauri {
	track_lists: HashMap<TrackListID, TrackList>,
	song_count: usize,
}

pub fn app_log_dir() -> Result<PathBuf> {
	#[cfg(target_os = "macos")]
	{
		use anyhow::Context;
		let home_dir = dirs_next::home_dir().context("Home folder not found")?;
		let log_dir = home_dir.join("Library/Logs").join("space.kasper.ferrum");
		return Ok(log_dir);
	}
	#[cfg(not(target_os = "macos"))]
	{
		use anyhow::Context;
		let local_data_dir = dirs_next::data_local_dir().context("Local data folder not found")?;
		let log_dir = local_data_dir.join("space.kasper.ferrum").join("logs");
		return Ok(log_dir);
	}
}

#[tauri::command]
#[specta::specta]
fn load_library(library_json: String, app: AppHandle) -> Result<LibraryTauri, String> {
	load_library_inner(library_json, app).map_err(|e| e.to_string())
}
fn load_library_inner(library_json: String, app: AppHandle) -> Result<LibraryTauri> {
	let is_dev = false;
	let local_data_path: Option<String> = None;
	let library_path: Option<String> = None;
	let mut library_dir;
	let cache_dir;
	let local_data_dir;
	if is_dev {
		let appdata_dev = env::current_dir().unwrap().join("src-native/appdata");
		library_dir = appdata_dev.join("Library");
		cache_dir = appdata_dev.join("Caches");
		local_data_dir = appdata_dev.join("LocalData/space.kasper.ferrum");
	} else {
		library_dir = dirs_next::audio_dir()
			.context("Music folder not found")?
			.join("Ferrum");
		cache_dir = dirs_next::cache_dir()
			.context("Cache folder not found")?
			.join("space.kasper.ferrum");
		local_data_dir = dirs_next::data_local_dir()
			.context("Local data folder not found")?
			.join("space.kasper.ferrum");
	};
	let local_data_dir = match local_data_path {
		Some(path) => PathBuf::from(path),
		None => local_data_dir,
	};
	if let Some(library_path) = library_path {
		library_dir = PathBuf::from(library_path);
	}
	// Not really used, just here to make migration work for now
	let paths = Paths {
		path_separator: std::path::MAIN_SEPARATOR_STR.into(),
		library_dir: path_to_string(&library_dir),
		tracks_dir: path_to_string(library_dir.join("Tracks")),
		library_json: path_to_string(library_dir.join("Library.json")),
		cache_dir: path_to_string(&cache_dir),
		cache_db: path_to_string(cache_dir.join("Cache.redb")),
		local_data_dir: path_to_string(&local_data_dir),
		view_options_file: path_to_string(local_data_dir.join("view.json")),
		queue_file: path_to_string(local_data_dir.join("queue.cbor")),
		// This makes sure we can get the logs dir, which is important for crash logs
		logs_dir: path_to_string(app_log_dir()?),
	};

	paths
		.ensure_dirs_exists()
		.context("Error ensuring folder exists")?;
	println!("Loading library at path: {}", paths.library_dir);

	let library = match load_library_from_file(&library_json, &paths) {
		Ok(library) => library,
		Err(err) => return Err(err),
	};

	let library_tauri = LibraryTauri {
		track_lists: library.trackLists.clone().into_iter().collect(),
		song_count: library.get_track_item_ids().len(),
	};

	app.manage(Mutex::new(library));

	Ok(library_tauri)
}

pub fn gen_types() -> Builder {
	let specta_builder =
		tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
			error_popup,
			load_library,
			open_file_persistent_android,
			get_track_by_item_id,
			get_tracks_page,
		]);

	#[cfg(all(debug_assertions, not(target_os = "android")))]
	specta_builder
		.export(
			specta_typescript::Typescript::default()
				.bigint(specta_typescript::BigIntExportBehavior::String),
			std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.parent() // if this Rust file is inside src-tauri
				.unwrap()
				.join("bindings.ts"),
		)
		.expect("Failed to export typescript bindings");
	specta_builder
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	#[cfg(target_os = "android")]
	android_logger::init_once(
		android_logger::Config::default()
			.with_max_level(log::LevelFilter::Trace)
			.with_tag("{{app.name}}"),
	);

	let specta_builder = gen_types();

	tauri::Builder::default()
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_android_fs::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_safe_area_insets_css::init())
		.invoke_handler(specta_builder.invoke_handler())
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
