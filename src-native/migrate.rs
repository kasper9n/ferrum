use crate::{
	library::{Paths, load_old_library_json},
	library_types::Library,
};
use anyhow::{Context, Result};
use sqlx::{
	ConnectOptions, Connection, Sqlite, migrate::MigrateDatabase, sqlite::SqliteConnectOptions,
};

pub async fn migrate_to_sqlite(paths: &Paths) -> Result<()> {
	let old_library = match load_old_library_json(&paths.library_json)? {
		None => {
			return Ok(());
		}
		Some(old_library) => old_library,
	};

	Sqlite::create_database(&paths.library_sqlite)
		.await
		.context("Could not create library database")?;

	let mut connection = SqliteConnectOptions::new()
		.filename(&paths.library_sqlite)
		.connect()
		.await
		.context("Error connecting to created library database")?;

	sqlx::migrate!("./src-native/migrations")
		.run(&mut connection)
		.await
		.context("Could not run database migrations")?;

	todo!("Save library to database");

	connection
		.close()
		.await
		.context("Could not save/close database")?;

	Ok(())
}
