use crate::data::Data;
use crate::library::Paths;
use crate::path_to_json;
use anyhow::{Context, Result};
use atomicwrites::AtomicFile;
use atomicwrites::OverwriteBehavior::AllowOverwrite;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[napi(object)]
pub struct ViewOptions {
	pub shown_playlist_folders: Vec<String>,
	/// Empty is treated as default
	pub columns: Vec<String>,
	/// Auto update checking
	#[serde(default)]
	pub no_auto_update: bool,
	pub skip_updating_to_version: Option<String>,
}
impl ViewOptions {
	pub fn load(paths: &Paths) -> ViewOptions {
		match path_to_json(&paths.view_options_file) {
			Ok(view_cache) => view_cache,
			Err(_) => ViewOptions {
				shown_playlist_folders: Vec::new(),
				columns: Vec::new(),
				no_auto_update: false,
				skip_updating_to_version: None,
			},
		}
	}
	pub fn save(&self, file_path: &str) -> Result<()> {
		let json_str = serde_json::to_string(self).context("Error saving view.json")?;
		let af = AtomicFile::new(&file_path, AllowOverwrite);
		af.write(|f| f.write_all(json_str.as_bytes()))
			.context("Error writing view.json")?;
		Ok(())
	}
}

#[napi(js_name = "load_view_options")]
#[allow(dead_code)]
pub fn load_view_options() -> Result<ViewOptions> {
	let data = Data::get_blocking();
	Ok(ViewOptions::load(&data.paths))
}
#[napi(js_name = "save_view_options")]
#[allow(dead_code)]
pub async fn save_view_options(view_options: ViewOptions, file_path: String) -> Result<()> {
	view_options.save(&file_path)?;
	Ok(())
}
