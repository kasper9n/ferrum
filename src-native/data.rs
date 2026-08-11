use crate::library::{Paths, load_library};
use crate::library_types::Library;
use crate::tracks::Tag;
use anyhow::Context;
use atomicwrites::{AllowOverwrite, AtomicFile};
use dirs_next;
use napi::Result;
use serde::Serialize;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant;
use tokio::sync::{Mutex, MutexGuard};

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
	path.as_ref()
		.to_str()
		.expect("Invalid path str")
		.to_string()
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

pub struct Data {
	pub paths: Paths,
	pub library: Library,
	/// Current tag being edited
	pub current_tag: Option<Tag>,
}

pub static DATA: OnceLock<Mutex<Data>> = OnceLock::new();

impl Data {
	pub async fn get_async() -> MutexGuard<'static, Data> {
		DATA.get().expect("No data initialised").lock().await
	}
	pub fn get_blocking() -> MutexGuard<'static, Data> {
		DATA.get().expect("No data initialised").blocking_lock()
	}
	pub fn save(&mut self) -> Result<()> {
		let mut now = Instant::now();
		let formatter = serde_json::ser::PrettyFormatter::with_indent(b"	"); // tab

		let mut json = Vec::new();
		let mut ser = serde_json::Serializer::with_formatter(&mut json, formatter);
		self.library.versioned().serialize(&mut ser)?;
		println!("Stringify: {}ms", now.elapsed().as_millis());

		now = Instant::now();
		let file_path = &self.paths.library_json;
		let af = AtomicFile::new(file_path, AllowOverwrite);
		af.write(|f| f.write_all(&json)).context("Error saving")?;
		println!("Write: {}ms", now.elapsed().as_millis());
		Ok(())
	}
	pub fn load(
		is_dev: bool,
		local_data_path: Option<String>,
		library_path: Option<String>,
	) -> Result<()> {
		if is_dev {
			println!("Starting in dev mode");
		}

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

		let loaded_library = load_library(&paths)?;

		let data = Data {
			paths,
			library: loaded_library,
			current_tag: None,
		};
		// if it fails, it was already set. the user might just have reloaded
		let _result = DATA.set(Mutex::new(data));
		return Ok(());
	}
}
