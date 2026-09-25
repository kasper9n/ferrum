use anyhow::{Context, Result};
use atomicwrites::{AtomicFile, OverwriteBehavior::AllowOverwrite};
use mimalloc::MiMalloc;
#[cfg(feature = "napi-rs")]
use serde::de::DeserializeOwned;
#[cfg(feature = "napi-rs")]
use std::fs::File;
#[cfg(feature = "napi-rs")]
use std::io::BufReader;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_os = "macos")]
use trash::macos::TrashContextExtMacos;

// Alloactor recommended by simd_json
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(feature = "napi-rs")]
#[macro_use]
extern crate napi_derive;

#[cfg(feature = "napi-rs")]
mod data;
#[cfg(feature = "napi-rs")]
mod data_js;
pub mod filter;
mod idvecmap;
#[cfg(feature = "napi-rs")]
mod itunes_import;
pub mod library;
pub mod library_types;
pub mod migrate;
pub mod page;
#[cfg(feature = "napi-rs")]
pub mod playlists;
mod queue_state;
pub mod sort;
pub mod test_data;
#[cfg(feature = "napi-rs")]
mod tracks;
#[cfg(feature = "napi-rs")]
mod view_options;

fn get_now_timestamp() -> i64 {
	let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => n.as_millis() as i64,
		Err(err) => err.duration().as_millis() as i64,
	};
	return timestamp;
}

#[cfg(feature = "napi-rs")]
fn sys_time_to_timestamp(sys_time: &SystemTime) -> i64 {
	let timestamp = match sys_time.duration_since(UNIX_EPOCH) {
		Ok(n) => n.as_millis() as i64,
		Err(err) => err.duration().as_millis() as i64,
	};
	return timestamp;
}

#[cfg(feature = "napi-rs")]
fn str_to_option(s: String) -> Option<String> {
	match s.as_str() {
		"" => None,
		_ => Some(s),
	}
}

#[cfg(feature = "napi-rs")]
fn path_to_json<J>(path: &str) -> Result<J>
where
	J: DeserializeOwned,
{
	let file = File::open(path).context("Error opening file")?;
	let reader = BufReader::new(file);
	let json = simd_json::from_reader(reader).context("Error parsing file")?;
	Ok(json)
}

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
	path.as_ref()
		.to_str()
		.expect("Invalid path str")
		.to_string()
}

pub fn save_overwrite<T: serde::Serialize>(value: &T, file_path: &String) -> Result<()> {
	let af = AtomicFile::new(file_path, AllowOverwrite);
	af.write(|f| {
		let mut f = BufWriter::with_capacity(1024 * 512, f);
		simd_json::serde::to_writer(&mut f, value).map_err(std::io::Error::other)
	})
	.context("Error saving")?;
	Ok(())
}

#[cfg(target_os = "android")]
pub fn delete_file(path: &PathBuf) -> Result<()> {
	use anyhow::bail;
	bail!("Unsupported");
}

#[cfg(not(target_os = "android"))]
pub fn delete_file(path: &PathBuf) -> Result<()> {
	#[allow(unused_mut)]
	let mut trash_context = trash::TrashContext::new();

	#[cfg(target_os = "macos")]
	trash_context.set_delete_method(trash::macos::DeleteMethod::NsFileManager);

	trash_context
		.delete(&path)
		.with_context(|| format!("Failed moving file to trash: {}", path.to_string_lossy()))
}
