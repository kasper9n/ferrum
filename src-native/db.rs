pub type TrackID = String;
pub type TrackListID = String;

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "lowercase")]
pub enum TrackListKind {
	Playlist,
	Folder,
	Special,
}
impl ToString for TrackListKind {
	fn to_string(&self) -> String {
		match self {
			TrackListKind::Playlist => "playlist".to_string(),
			TrackListKind::Folder => "folder".to_string(),
			TrackListKind::Special => "special".to_string(),
		}
	}
}

// #[derive(sqlx::FromRow, Debug)]
// pub struct TrackList {
// 	id: String,
// 	kind: String,
// 	parent_id: Option<TrackListID>,
// 	item_index: Option<i64>,
// 	name: String,
// 	description: String,
// 	liked: bool,
// 	disliked: bool,
// 	/// For example "itunes"
// 	imported_from: Option<String>,
// 	/// For example iTunes Persistent ID
// 	original_id: Option<String>,
// 	imported_at: Option<i64>,
// 	/// Nullable for imported playlists
// 	created_at: Option<i64>,
// }

// let tracklist: db::TrackList = sqlx::query_as("SELECT * FROM track_lists WHERE id = ?")
// 		.bind(&options.playlist_id)
// 		.fetch_one(&mut data.db)
// 		.await?;
