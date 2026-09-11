use anyhow::{Context, Result};
use atomicwrites::{AtomicFile, OverwriteBehavior::AllowOverwrite};
use mimalloc::MiMalloc;
use serde::Serialize;
#[cfg(feature = "napi-rs")]
use serde::de::DeserializeOwned;
#[cfg(feature = "napi-rs")]
use std::fs::File;
#[cfg(feature = "napi-rs")]
use std::io::BufReader;
use std::{
	io::Write,
	path::Path,
	time::{Instant, SystemTime, UNIX_EPOCH},
};

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
	let json = serde_json::from_reader(reader).context("Error parsing file")?;
	Ok(json)
}

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
	path.as_ref()
		.to_str()
		.expect("Invalid path str")
		.to_string()
}

pub fn serialize_json_pretty<S: Serialize>(value: &S) -> Result<Vec<u8>> {
	let now = Instant::now();
	let formatter = serde_json::ser::PrettyFormatter::with_indent(b"	"); // tab
	let mut json = Vec::new();
	let mut ser = serde_json::Serializer::with_formatter(&mut json, formatter);
	value.serialize(&mut ser)?;
	println!("Stringify: {}ms", now.elapsed().as_millis());
	Ok(json)
}

pub fn save_overwrite(bytes: Vec<u8>, file_path: &String) -> Result<()> {
	let now = Instant::now();
	let af = AtomicFile::new(file_path, AllowOverwrite);
	af.write(|f| f.write_all(&bytes)).context("Error saving")?;
	println!("Write: {}ms", now.elapsed().as_millis());
	Ok(())
}
