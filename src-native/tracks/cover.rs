use super::Tag;
use anyhow::{Context, Result, anyhow, bail};
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, ImageFormat, ImageReader};
#[cfg(feature = "napi")]
use napi::bindgen_prelude::Buffer;
use redb::{Database, TableDefinition};
use std::fs;
use std::io::{BufWriter, Cursor};
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use std::time::{Instant, UNIX_EPOCH};

// (modified_timestamp_ms, image_bytes)
type CacheEntry = (i64, Vec<u8>);

const IMG_CACHE_TABLE: TableDefinition<&str, CacheEntry> = TableDefinition::new("img_cache");

static CACHE_DB: LazyLock<RwLock<Option<Database>>> = LazyLock::new(|| RwLock::new(None));

fn init_cache_db(path: String) -> Result<()> {
	let now = Instant::now();

	// Check quickly with a read-lock first
	{
		let cache_db_global = &*CACHE_DB;
		let cache_db_lock = cache_db_global.read().unwrap();
		if cache_db_lock.is_some() {
			return Ok(());
		}
	}

	let cache_db_global = &*CACHE_DB;
	let mut cache_db_lock = cache_db_global.write().unwrap();
	if cache_db_lock.is_none() {
		let db = Database::create(&path).context("Could not load image cache")?;
		let init_txn = db
			.begin_write()
			.context("Could not begin write transaction")?;
		{
			// Create table
			init_txn
				.open_table(IMG_CACHE_TABLE)
				.context("Could not open table")?;
		}
		init_txn.commit().context("Could not commit cache")?;
		*cache_db_lock = Some(db);
		println!("Initialized Cache.redb: {}ms", now.elapsed().as_millis());
	}
	Ok(())
}

#[cfg(feature = "napi")]
#[napi(js_name = "close_cache_db")]
#[allow(dead_code)]
pub async fn close_cache_db() -> napi::Result<()> {
	let cache_db_global = &*CACHE_DB;
	let mut cache_db_lock = cache_db_global.write().unwrap();
	if let Some(cache_db) = cache_db_lock.take() {
		let now = Instant::now();
		drop(cache_db);
		println!("Closed Cache.redb: {}ms", now.elapsed().as_millis());
	} else {
		// This can happen if the app crashes during startup
		println!("Cache.redb was not open");
	}

	Ok(())
}

/// Returns `None` if the file does not have an image
fn get_modified_timestamp_ms(path: &str) -> Result<Option<u128>> {
	let file_metadata = match fs::metadata(path) {
		Ok(file_metadata) => file_metadata,
		Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
			return Ok(None);
		}
		Err(err) => {
			return Err(err).context("Error reading file metadata of {path}")?;
		}
	};
	let modified_timestamp_ms = match file_metadata.modified() {
		Ok(modified_time) => modified_time
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_millis(),
		Err(_) => {
			println!("Not caching cover due to platform lacking support for modified timestamps.");
			return Ok(None);
		}
	};
	return Ok(Some(modified_timestamp_ms));
}

fn get_cached_image(
	cache_db: &Database,
	path: &str,
	date_modified_ms: i64,
) -> Result<Option<Vec<u8>>> {
	let read_txn = cache_db
		.begin_read()
		.context("Could not begin read transaction")?;
	let cache_entry = {
		let table = read_txn
			.open_table(IMG_CACHE_TABLE)
			.context("Could not open table")?;
		let record = table.get(path).context("Could not get record")?;
		match record {
			Some(cache_entry) => cache_entry.value(),
			None => return Ok(None),
		}
	};

	if date_modified_ms == cache_entry.0 {
		return Ok(Some(cache_entry.1));
	}
	Ok(None)
}

fn write_to_cache(cache_db: &Database, path: &str, value: CacheEntry) -> Result<()> {
	let write_txn = cache_db
		.begin_write()
		.context("Could not begin write transaction")?;
	{
		let mut table = write_txn
			.open_table(IMG_CACHE_TABLE)
			.context("Could not open table")?;
		table
			.insert(&*path, value)
			.context("Could not insert record")?;
	}
	write_txn.commit().context("Could not commit transaction")?;
	Ok(())
}

/// Returns `None` if the file does not have an image
#[cfg(feature = "napi")]
#[napi(js_name = "read_small_cover_async")]
#[allow(dead_code)]
pub async fn read_small_cover_async(
	path: String,
	index: u16,
	cache_db_path: String,
) -> napi::Result<Option<Buffer>> {
	if path == "" {
		return Err(anyhow!("path must not be empty").into());
	} else if cache_db_path == "" {
		return Err(anyhow!("cache_db_path must not be empty").into());
	}

	init_cache_db(cache_db_path)?;

	let cache_db_global = &*CACHE_DB;
	let cache_db_lock = cache_db_global.read().unwrap();
	let cache_db = cache_db_lock.as_ref().unwrap();

	let date_modified_ms: Option<i64> =
		get_modified_timestamp_ms(&path)?.map(|n| n.try_into().unwrap());

	if let Some(date_modified_ms) = date_modified_ms {
		if let Some(img_bytes) = get_cached_image(cache_db, &path, date_modified_ms)? {
			return Ok(Some(img_bytes.into()));
		}
	}

	let tag = Tag::read_from_path(&PathBuf::from(path.clone()))?;
	let image = match tag.get_image_consume(index.into())? {
		Some(image) => image,
		None => return Ok(None),
	};

	let img_bytes = to_resized_image(image.data, 84)?;

	if let Some(date_modified_ms) = date_modified_ms {
		let value = (date_modified_ms, img_bytes.clone());
		write_to_cache(cache_db, &path, value)?;
	}

	Ok(Some(img_bytes.into()))
}

fn to_resized_image(image_bytes: Vec<u8>, max_size: u32) -> Result<Vec<u8>> {
	let image = ImageReader::new(Cursor::new(image_bytes))
		.with_guessed_format()
		.context("Unable to guess image format")?;
	let format_format = image.format();
	let src_image = image.decode().context("Unable to decode image")?;

	let src_width = src_image.width();
	let src_height = src_image.height();

	let mut dst_width = max_size.min(src_width);
	let mut dst_height = max_size.min(src_height);
	if src_width > src_height {
		dst_width = max_size;
		dst_height = max_size * src_height / src_width;
	} else if src_height > src_width {
		dst_height = max_size;
		dst_width = max_size * src_width / src_height;
	}
	let mut dst_image = Image::new(dst_width, dst_height, src_image.pixel_type().unwrap());

	let mut resizer = Resizer::new();
	resizer.resize(&src_image, &mut dst_image, None).unwrap();

	let mut result_buf = BufWriter::new(Vec::new());
	let img_bytes = match format_format {
		Some(ImageFormat::Jpeg) => {
			JpegEncoder::new(&mut result_buf)
				.write_image(
					dst_image.buffer(),
					dst_width,
					dst_height,
					src_image.color().into(),
				)
				.context("Unable to encode image")?;
			result_buf
				.into_inner()
				.context("Error getting inner img buffer")?
		}
		Some(ImageFormat::Png) => {
			PngEncoder::new(&mut result_buf)
				.write_image(
					dst_image.buffer(),
					dst_width,
					dst_height,
					src_image.color().into(),
				)
				.context("Unable to encode image")?;
			result_buf
				.into_inner()
				.context("Error getting inner img buffer")?
		}
		_ => bail!("Unsupported image type"),
	};
	Ok(img_bytes)
}

#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi-rs", napi(js_name = "read_cover_async"))]
#[allow(dead_code)]
pub async fn read_cover_async(file_path: String, index: u16) -> Result<Option<Buffer>> {
	let file_path = PathBuf::from(file_path);
	let tag = Tag::read_from_path(&file_path)?;
	let image = match tag.get_image_consume(index.into())? {
		Some(image) => image,
		None => return Ok(None),
	};
	Ok(Some(Buffer::from(image.data)))
}
