use std::{env, path::PathBuf};
extern crate napi_build;

fn main() {
	// Path to SQLite file relative to the crate root
	let db_path = PathBuf::from("src-native/appdata/Library/Library-typegen.sqlite");
	let abs_path = env::current_dir().unwrap().join(&db_path);
	let database_url = format!("sqlite:///{}", abs_path.display());
	println!("cargo:rustc-env=DATABASE_URL={}", database_url);

	#[cfg(not(target_os = "android"))]
	napi_build::setup();
}
