use anyhow::{Context, Result};
use mimalloc::MiMalloc;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::BufReader;
use std::time::{SystemTime, UNIX_EPOCH};

// Alloactor recommended by simd_json
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(feature = "napi")]
#[macro_use]
extern crate napi_derive;

mod data;
#[cfg(feature = "napi")]
mod data_js;
pub mod db;
pub mod filter;
#[cfg(feature = "napi")]
mod itunes_import;
pub mod library;
pub mod library_types;
mod migrate;
pub mod page;
#[cfg(feature = "napi")]
pub mod playlists;
#[cfg(feature = "napi")]
mod queue_state;
pub mod sort;
mod tracks;
#[cfg(feature = "napi")]
mod view_options;

fn get_now_timestamp() -> i64 {
	let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => n.as_millis() as i64,
		Err(err) => err.duration().as_millis() as i64,
	};
	return timestamp;
}

fn sys_time_to_timestamp(sys_time: &SystemTime) -> i64 {
	let timestamp = match sys_time.duration_since(UNIX_EPOCH) {
		Ok(n) => n.as_millis() as i64,
		Err(err) => err.duration().as_millis() as i64,
	};
	return timestamp;
}

fn str_to_option(s: String) -> Option<String> {
	match s.as_str() {
		"" => None,
		_ => Some(s),
	}
}

fn path_to_json<J>(path: &str) -> Result<J>
where
	J: DeserializeOwned,
{
	let file = File::open(path).context("Error opening file")?;
	let reader = BufReader::new(file);
	let json = serde_json::from_reader(reader).context("Error parsing file")?;
	Ok(json)
}
