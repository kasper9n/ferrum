use anyhow::{Context, Result};
use atomicwrites::AtomicFile;
use atomicwrites::OverwriteBehavior::AllowOverwrite;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "napi", napi(object))]
pub struct QueueItemState {
	#[serde(rename = "qId")]
	#[napi(js_name = "qId")]
	pub q_id: i64,
	pub id: String,
	#[napi(js_name = "non_shuffle_pos")]
	pub non_shuffle_pos: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "napi", napi(object))]
pub struct QueueCurrentState {
	pub item: QueueItemState,
	#[napi(js_name = "from_auto_queue")]
	pub from_auto_queue: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "napi", napi(object))]
pub struct QueueState {
	#[serde(default)]
	pub past: Vec<QueueItemState>,
	#[serde(default)]
	pub current: Option<QueueCurrentState>,
	#[serde(default)]
	#[napi(js_name = "user_queue")]
	pub user_queue: Vec<QueueItemState>,
	#[serde(default)]
	#[napi(js_name = "auto_queue")]
	pub auto_queue: Vec<QueueItemState>,
	#[serde(default)]
	#[napi(js_name = "last_qid")]
	pub last_qid: i64,
	#[serde(default)]
	pub shuffle: bool,
	#[serde(default)]
	pub repeat: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum DiskAutoQueueItem {
	Id(String),
	IdAndPos((String, u32)),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct DiskQueueState(
	Vec<String>,            // past
	Option<String>,         // current
	Vec<String>,            // user_queue
	Vec<DiskAutoQueueItem>, // auto_queue
	bool,                   // shuffle
	bool,                   // repeat
);

impl From<&QueueState> for DiskQueueState {
	fn from(value: &QueueState) -> Self {
		DiskQueueState(
			value.past.iter().map(|item| item.id.clone()).collect(),
			value
				.current
				.as_ref()
				.map(|current| current.item.id.clone()),
			value
				.user_queue
				.iter()
				.map(|item| item.id.clone())
				.collect(),
			value
				.auto_queue
				.iter()
				.map(|item| match item.non_shuffle_pos {
					Some(non_shuffle_pos) => {
						DiskAutoQueueItem::IdAndPos((item.id.clone(), non_shuffle_pos))
					}
					None => DiskAutoQueueItem::Id(item.id.clone()),
				})
				.collect(),
			value.shuffle,
			value.repeat,
		)
	}
}

impl From<DiskQueueState> for QueueState {
	fn from(value: DiskQueueState) -> Self {
		let DiskQueueState(past_ids, current_id, user_ids, auto_items, shuffle, repeat) = value;

		let mut next_qid: i64 = -1;
		let mut new_item = |id: String, non_shuffle_pos: Option<u32>| {
			next_qid += 1;
			QueueItemState {
				q_id: next_qid,
				id,
				non_shuffle_pos,
			}
		};

		let past = past_ids
			.into_iter()
			.map(|id| new_item(id, None))
			.collect::<Vec<_>>();
		let current = current_id.map(|id| QueueCurrentState {
			item: new_item(id, None),
			from_auto_queue: false,
		});
		let user_queue = user_ids
			.into_iter()
			.map(|id| new_item(id, None))
			.collect::<Vec<_>>();
		let auto_queue = auto_items
			.into_iter()
			.map(|item| match item {
				DiskAutoQueueItem::Id(id) => new_item(id, None),
				DiskAutoQueueItem::IdAndPos((id, non_shuffle_pos)) => {
					new_item(id, Some(non_shuffle_pos))
				}
			})
			.collect::<Vec<_>>();

		QueueState {
			past,
			current,
			user_queue,
			auto_queue,
			last_qid: next_qid,
			shuffle,
			repeat,
		}
	}
}

impl QueueState {
	pub fn load(file_path: &str) -> Option<QueueState> {
		let bytes = fs::read(file_path).ok()?;
		if let Ok(compact) = serde_cbor::from_slice::<DiskQueueState>(&bytes) {
			return Some(compact.into());
		}
		// Backward compatibility for existing full-struct CBOR
		serde_cbor::from_slice::<QueueState>(&bytes).ok()
	}

	pub fn save(&self, file_path: &str) -> Result<()> {
		let compact: DiskQueueState = self.into();
		let bytes = serde_cbor::to_vec(&compact).context("Error encoding queue.cbor")?;
		let af = AtomicFile::new(file_path, AllowOverwrite);
		af.write(|f| f.write_all(&bytes))
			.context("Error writing queue.cbor")?;
		Ok(())
	}
}

#[napi(js_name = "load_queue_state")]
#[allow(dead_code)]
pub fn load_queue_state(file_path: String) -> Option<QueueState> {
	QueueState::load(&file_path)
}

#[napi(js_name = "save_queue_state")]
#[allow(dead_code)]
pub async fn save_queue_state(queue_state: QueueState, file_path: String) -> Result<()> {
	queue_state.save(&file_path)?;
	Ok(())
}
