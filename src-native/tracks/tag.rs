use anyhow::{Context, Result, bail};
use lofty::picture::{MimeType, Picture};
use lofty::tag::ItemKey;
use lofty::{file::TaggedFileExt, tag::Accessor, tag::TagExt};
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub struct Image {
	pub data: Vec<u8>,
}
pub struct ImageRef<'a> {
	// i64 because napi doesn't support u64
	pub index: i64,
	pub total_images: i64,
	pub mime_type: MimeType,
	pub data: &'a [u8],
}

pub struct Tag {
	tag: lofty::tag::Tag,
}
impl Tag {
	pub fn read_from_path(path: &PathBuf) -> Result<Tag> {
		if !path.exists() {
			bail!("File does not exist: {}", path.to_string_lossy());
		}
		let ext = path.extension().unwrap_or_default().to_str().unwrap();

		let tag = match ext.as_ref() {
			"mp3" | "m4a" | "opus" => {
				let parse_options = lofty::config::ParseOptions::new()
					.read_properties(false)
					.parsing_mode(lofty::config::ParsingMode::Strict);
				let probe = lofty::probe::Probe::open(path)
					.context("File does not exist")?
					.options(parse_options);

				let mut tagged_file = probe.read().context("Unable to read file")?;

				let tag = match tagged_file.remove(tagged_file.primary_tag_type()) {
					Some(t) => t.clone(),
					None => lofty::tag::Tag::new(tagged_file.primary_tag_type()),
				};

				Tag { tag }
			}
			_ => bail!("Unsupported file extension: {}", ext),
		};
		Ok(tag)
	}
	pub fn write_to_path(&mut self, path: &Path) -> Result<()> {
		self.tag
			.save_to_path(path, lofty::config::WriteOptions::default())
			.context("Unable to tag file")
	}
	pub fn remove_title(&mut self) {
		self.tag.remove_title()
	}
	pub fn set_title(&mut self, value: &str) {
		self.tag.set_title(value.to_string())
	}
	pub fn remove_artists(&mut self) {
		self.tag.remove_artist()
	}
	pub fn set_artist(&mut self, value: &str) {
		self.tag.set_artist(value.to_string())
	}
	pub fn remove_album(&mut self) {
		self.tag.remove_album()
	}
	pub fn set_album(&mut self, value: &str) {
		self.tag.set_album(value.to_string())
	}
	pub fn remove_album_artists(&mut self) {
		self.tag.remove_key(&ItemKey::AlbumArtist);
	}
	pub fn set_album_artist(&mut self, value: &str) {
		let inserted = self
			.tag
			.insert_text(ItemKey::AlbumArtist, value.to_string());
		assert!(inserted, "Failed to set album artist");
	}
	pub fn remove_composers(&mut self) {
		self.tag.remove_key(&ItemKey::Composer);
	}
	pub fn set_composer(&mut self, value: &str) {
		let inserted = self.tag.insert_text(ItemKey::Composer, value.to_string());
		assert!(inserted, "Failed to set composer");
	}
	pub fn remove_groupings(&mut self) {
		self.tag.remove_key(&ItemKey::ContentGroup)
	}
	pub fn set_grouping(&mut self, value: &str) {
		let inserted = self
			.tag
			.insert_text(ItemKey::ContentGroup, value.to_string());
		assert!(inserted, "Failed to set grouping");
	}
	pub fn remove_genres(&mut self) {
		self.tag.remove_genre()
	}
	pub fn set_genre(&mut self, value: &str) {
		self.tag.set_genre(value.to_string())
	}
	pub fn remove_year(&mut self) {
		self.tag.remove_year()
	}
	pub fn set_year(&mut self, value: i32) {
		let u = if value < 0 { 0 } else { value as u32 };
		self.tag.set_year(u);
	}
	pub fn set_track_info(&mut self, number: Option<u32>, total: Option<u32>) -> () {
		match number {
			Some(number) => self.tag.set_track(number),
			None => self.tag.remove_track(),
		}
		match total {
			Some(total) => self.tag.set_track_total(total),
			None => self.tag.remove_track_total(),
		}
	}
	pub fn set_disc_info(&mut self, number: Option<u32>, total: Option<u32>) -> () {
		match number {
			Some(number) => self.tag.set_disk(number),
			None => self.tag.remove_disk(),
		}
		match total {
			Some(total) => self.tag.set_disk_total(total),
			None => self.tag.remove_disk_total(),
		}
	}
	pub fn remove_bpm(&mut self) {
		self.tag.remove_key(&ItemKey::Bpm);
	}
	pub fn set_bpm(&mut self, value: u16) {
		let mut inserted = self.tag.insert_text(ItemKey::Bpm, value.to_string());
		if !inserted {
			inserted = self.tag.insert_text(ItemKey::IntegerBpm, value.to_string());
		}
		assert!(inserted, "Failed to set BPM");
	}
	pub fn remove_comments(&mut self) {
		self.tag.remove_comment()
	}
	pub fn set_comment(&mut self, value: &str) {
		self.tag.set_comment(value.to_string())
	}
	pub fn set_image(&mut self, index: usize, data: Vec<u8>) -> Result<()> {
		let mut reader = Cursor::new(data);
		let picture =
			lofty::picture::Picture::from_reader(&mut reader).context("Unable to read picture")?;
		match picture.mime_type() {
			Some(lofty::picture::MimeType::Png | lofty::picture::MimeType::Jpeg) => {
				self.tag.set_picture(index, picture);
			}
			_ => bail!("Unsupported picture type"),
		}

		Ok(())
	}
	pub fn get_image_ref(&'_ self, index: usize) -> Result<Option<ImageRef<'_>>> {
		let pictures = self.tag.pictures();
		match pictures.get(index) {
			Some(pic) => {
				let data = pic.data();
				Ok(Some(ImageRef {
					index: index.try_into().expect("usize conv"),
					total_images: pictures.len().try_into().expect("usize conv"),
					data,
					mime_type: pic.mime_type().context("No mime type")?.clone(),
				}))
			}
			None => Ok(None),
		}
	}
	pub fn get_image_consume(mut self, index: usize) -> Result<Option<Image>> {
		if self.tag.picture_count() <= index.try_into().expect("usize conv") {
			return Ok(None);
		}
		let pic = self.tag.remove_picture(index);
		Ok(Some(Image {
			data: pic.into_data(),
		}))
	}
	pub fn remove_image(&mut self, index: usize) -> Picture {
		self.tag.remove_picture(index)
	}
}
