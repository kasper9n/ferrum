use anyhow::Result;
use linked_hash_map::LinkedHashMap;
use num_traits::{CheckedAdd, One, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Display, hash::Hash, ops::Deref};

pub trait HasId<K> {
	fn id(&self) -> K;
}

pub trait UnsignedInt:
	Copy + Eq + Hash + Ord + Display + Zero + One + CheckedAdd + 'static
{
}
impl<T> UnsignedInt for T where
	T: Copy + Eq + Hash + Ord + Display + Zero + One + CheckedAdd + 'static
{
}

/// A LinkedHashMap<V> that's always keyed by V's own id, and (de)serializes
/// as a plain array on the wire instead of an object.
#[derive(Clone, Debug)]
pub struct IdMap<K, V>
where
	K: Hash + Eq + UnsignedInt,
{
	last_id: Option<K>,
	map: LinkedHashMap<K, V>,
}

impl<K, V> IdMap<K, V>
where
	K: Hash + Eq + UnsignedInt,
	V: HasId<K>,
{
	pub fn new() -> Self {
		Self {
			last_id: None,
			map: LinkedHashMap::new(),
		}
	}
	/// Creates an empty linked hash map with the given initial capacity.
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			map: LinkedHashMap::with_capacity(capacity),
			last_id: None,
		}
	}
	pub fn get_next_id(&self) -> K {
		match self.last_id {
			Some(last_id) => last_id.checked_add(&K::one()).unwrap(),
			None => {
				let last_id = self.map.iter().map(|(id, _)| id).max();
				match last_id {
					Some(last_id) => last_id.checked_add(&K::one()).unwrap(),
					None => K::zero(),
				}
			}
		}
	}
	pub fn try_insert(&mut self, id: K, value: V) -> &mut V {
		let new_last_id = match self.last_id {
			Some(last_id) => last_id.max(id),
			None => {
				let last_id = self.map.iter().map(|(id, _)| id).max();
				match last_id {
					Some(last_id) => (*last_id).max(id),
					None => id,
				}
			}
		};
		match self.map.entry(id) {
			linked_hash_map::Entry::Occupied(_) => panic!("ID {id} already exists"),
			linked_hash_map::Entry::Vacant(vacant) => {
				self.last_id = Some(new_last_id);
				vacant.insert(value)
			}
		}
	}
	pub fn get_mut(&mut self, id: &K) -> Option<&mut V> {
		self.map.get_mut(id)
	}

	pub fn remove(&mut self, id: &K) -> Option<V> {
		self.map.remove(id)
	}
}

// Read-only access falls through to the inner map (get, values, iter, len, contains_key, ...)
impl<K, V> Deref for IdMap<K, V>
where
	K: Hash + Eq + UnsignedInt,
{
	type Target = LinkedHashMap<K, V>;
	fn deref(&self) -> &Self::Target {
		&self.map
	}
}

impl<K, V> IntoIterator for IdMap<K, V>
where
	K: Hash + Eq + UnsignedInt,
{
	type Item = (K, V);
	type IntoIter = linked_hash_map::IntoIter<K, V>;
	fn into_iter(self) -> Self::IntoIter {
		self.map.into_iter()
	}
}

impl<K, V> Serialize for IdMap<K, V>
where
	K: Hash + Eq + UnsignedInt,
	V: Serialize,
{
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let vec: Vec<&V> = self.map.values().collect();
		vec.serialize(serializer)
	}
}

impl<'de, K, V> Deserialize<'de> for IdMap<K, V>
where
	K: Hash + Eq + UnsignedInt,
	V: Deserialize<'de> + HasId<K>,
{
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let vec: Vec<V> = Vec::deserialize(deserializer)?;
		let mut map = IdMap::with_capacity(vec.len());
		for item in vec {
			map.try_insert(item.id(), item);
		}
		Ok(map)
	}
}
