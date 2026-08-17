use crate::data::{Data, app_log_dir, path_to_string};
use crate::library::Paths;
use anyhow::{Context, Result};
use std::fs;
use tokio::runtime::Runtime;

#[napi(js_name = "load_data")]
#[allow(dead_code)]
pub fn load_data(
	is_dev: bool,
	local_data_path: Option<String>,
	library_path: Option<String>,
) -> Result<()> {
	std::panic::set_hook(Box::new(move |info| {
		let backtrace = std::backtrace::Backtrace::force_capture();
		let log_msg = format!("{info}\n\nBacktrace:\n{backtrace}");
		eprintln!("{}", log_msg);

		let logs_dir = app_log_dir().unwrap();
		fs::create_dir_all(&logs_dir).unwrap();

		let filename = format!(
			"Crash {}.log",
			chrono::Local::now().format("%Y-%m-%d %H-%M-%S")
		);
		let file_path = logs_dir.join(filename);

		fs::write(&file_path, log_msg).expect("Could not save crash log");
		println!("Crash message written to {}", file_path.to_string_lossy());
	}));
	let rt = Runtime::new().context("Error creating tokio runtime")?;

	rt.block_on(Data::load(is_dev, local_data_path, library_path))?;

	return Ok(());
}

#[napi(js_name = "get_paths")]
#[allow(dead_code)]
pub fn get_paths() -> Paths {
	let data = Data::get_blocking();
	data.paths.clone()
}
#[napi(js_name = "get_logs_dir")]
#[allow(dead_code)]
pub fn get_logs_dir() -> Result<String> {
	match app_log_dir() {
		Ok(path) => Ok(path_to_string(path)),
		Err(err) => Err(err),
	}
}

#[napi(js_name = "save")]
#[allow(dead_code)]
pub fn save() -> Result<()> {
	let mut data = Data::get_blocking();
	data.save()?;
	Ok(())
}
