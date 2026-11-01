use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct RecordStore {
    pub name: String,
    pub records: HashMap<i32, Vec<u8>>,
    pub next_id: i32,
    pub version: i32,
    pub last_modified: u64,
}

impl RecordStore {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            records: HashMap::new(),
            next_id: 1,
            version: 1,
            last_modified: 0,
        }
    }

    pub fn add_record(&mut self, data: &[u8]) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.records.insert(id, data.to_vec());
        self.version += 1;
        id
    }

    pub fn get_record(&self, id: i32) -> Option<&[u8]> {
        self.records.get(&id).map(|v| v.as_slice())
    }

    pub fn set_record(&mut self, id: i32, data: &[u8]) -> bool {
        if self.records.contains_key(&id) {
            self.records.insert(id, data.to_vec());
            self.version += 1;
            true
        } else {
            false
        }
    }

    pub fn delete_record(&mut self, id: i32) -> bool {
        if self.records.remove(&id).is_some() {
            self.version += 1;
            true
        } else {
            false
        }
    }

    /// Record ids in ascending order, as an enumeration must yield them.
    pub fn ids(&self) -> Vec<i32> {
        let mut ids: Vec<i32> = self.records.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    pub fn get_num_records(&self) -> i32 {
        self.records.len() as i32
    }

    pub fn get_next_record_id(&self) -> i32 {
        self.next_id
    }

    pub fn get_size(&self) -> i32 {
        let mut total = 0;
        for r in self.records.values() {
            total += r.len() as i32;
        }
        total
    }
}

#[derive(Debug, Clone, Default)]
pub struct RmsManager {
    pub stores: HashMap<String, RecordStore>,
}

impl RmsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, name: &str, create: bool) -> Result<&mut RecordStore, String> {
        if !self.stores.contains_key(name) {
            if create {
                self.stores.insert(name.to_string(), RecordStore::new(name));
            } else {
                return Err(format!("RecordStoreNotFound: {}", name));
            }
        }
        Ok(self.stores.get_mut(name).unwrap())
    }

    /// Names of currently open stores, for diagnostics.
    pub fn names(&self) -> Vec<String> {
        self.stores.keys().cloned().collect()
    }

    pub fn delete(&mut self, name: &str) -> bool {
        self.stores.remove(name).is_some()
    }

    pub fn list(&self) -> Vec<String> {
        self.stores.keys().cloned().collect()
    }

    /// Binary serialization of all stores for IndexedDB / localStorage
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Magic
        buf.extend_from_slice(b"RMS1");
        // Count of stores
        buf.extend_from_slice(&(self.stores.len() as u32).to_le_bytes());

        for (name, store) in &self.stores {
            let name_bytes = name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(name_bytes);
            buf.extend_from_slice(&(store.next_id).to_le_bytes());
            buf.extend_from_slice(&(store.records.len() as u32).to_le_bytes());

            for (&id, data) in &store.records {
                buf.extend_from_slice(&id.to_le_bytes());
                buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
                buf.extend_from_slice(data);
            }
        }
        buf
    }

    /// Binary deserialization
    pub fn deserialize(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() < 8 || &bytes[..4] != b"RMS1" {
            return Err("Invalid RMS serialization header".to_string());
        }
        let mut pos = 4;
        let num_stores =
            u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
                as usize;
        pos += 4;

        for _ in 0..num_stores {
            if pos + 2 > bytes.len() {
                break;
            }
            let name_len = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]) as usize;
            pos += 2;
            if pos + name_len > bytes.len() {
                break;
            }
            let name = String::from_utf8_lossy(&bytes[pos..pos + name_len]).to_string();
            pos += name_len;

            if pos + 8 > bytes.len() {
                break;
            }
            let next_id =
                i32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);
            pos += 4;
            let num_records =
                u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
                    as usize;
            pos += 4;

            let mut store = RecordStore::new(&name);
            store.next_id = next_id;

            for _ in 0..num_records {
                if pos + 8 > bytes.len() {
                    break;
                }
                let rec_id = i32::from_le_bytes([
                    bytes[pos],
                    bytes[pos + 1],
                    bytes[pos + 2],
                    bytes[pos + 3],
                ]);
                pos += 4;
                let rec_len = u32::from_le_bytes([
                    bytes[pos],
                    bytes[pos + 1],
                    bytes[pos + 2],
                    bytes[pos + 3],
                ]) as usize;
                pos += 4;
                if pos + rec_len > bytes.len() {
                    break;
                }
                let data = bytes[pos..pos + rec_len].to_vec();
                pos += rec_len;
                store.records.insert(rec_id, data);
            }

            self.stores.insert(name, store);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_creates_store_when_requested() {
        let mut rms = RmsManager::new();
        assert!(rms.open("data1", true).is_ok());
        assert_eq!(rms.names(), vec!["data1".to_string()]);
    }

    #[test]
    fn open_without_create_fails_on_missing_store() {
        let mut rms = RmsManager::new();
        assert!(rms.open("data1", false).is_err());
    }

    #[test]
    fn records_are_enumerable_in_id_order() {
        // Regression: RecordStore.enumerateRecords was missing entirely, so
        // RecordEnumeration.numRecords() NPE'd on a null receiver and killed
        // the game thread. The store itself must expose a stable, ordered id
        // list for the enumeration to iterate.
        let mut rms = RmsManager::new();
        let store = rms.open("data1", true).unwrap();
        let a = store.add_record(b"alpha");
        let b = store.add_record(b"beta");
        let c = store.add_record(b"gamma");
        assert_eq!(store.get_num_records(), 3);

        assert_eq!(store.ids(), vec![a, b, c]);

        assert_eq!(store.get_record(a), Some(&b"alpha"[..]));
        assert_eq!(store.get_record(c), Some(&b"gamma"[..]));
    }

    #[test]
    fn deleting_a_record_removes_it_from_enumeration() {
        let mut rms = RmsManager::new();
        let store = rms.open("data1", true).unwrap();
        let a = store.add_record(b"x");
        let b = store.add_record(b"y");
        assert!(store.delete_record(a));
        assert_eq!(store.ids(), vec![b]);
    }

    #[test]
    fn test_rms_record_id_lifecycle_never_reuses_deleted_id() {
        let mut rms = RmsManager::new();
        let store = rms.open("game_save", true).unwrap();

        // 1. Empty store has next_id = 1
        assert_eq!(store.get_next_record_id(), 1);
        assert_eq!(store.get_num_records(), 0);

        // 2. Add 3 records: IDs must be 1, 2, 3
        let id1 = store.add_record(b"rec1");
        assert_eq!(id1, 1);
        assert_eq!(store.get_next_record_id(), 2);

        let id2 = store.add_record(b"rec2");
        assert_eq!(id2, 2);
        assert_eq!(store.get_next_record_id(), 3);

        let id3 = store.add_record(b"rec3");
        assert_eq!(id3, 3);
        assert_eq!(store.get_next_record_id(), 4);
        assert_eq!(store.get_num_records(), 3);

        // 3. Delete middle record 2: next_id must remain 4 (never reused!)
        assert!(store.delete_record(id2));
        assert_eq!(store.get_num_records(), 2);
        assert_eq!(store.get_next_record_id(), 4);

        // 4. Delete last record 3: next_id must still remain 4!
        assert!(store.delete_record(id3));
        assert_eq!(store.get_num_records(), 1);
        assert_eq!(store.get_next_record_id(), 4);

        // 5. Delete all records without deleting the store: next_id does not reset
        assert!(store.delete_record(id1));
        assert_eq!(store.get_num_records(), 0);
        assert_eq!(store.get_next_record_id(), 4);

        // 6. Adding a new record after deletions gets next_id 4
        let id4 = store.add_record(b"rec4");
        assert_eq!(id4, 4);
        assert_eq!(store.get_next_record_id(), 5);

        // 7. Delete store and create new store: starts sequence from 1
        assert!(rms.delete("game_save"));
        let new_store = rms.open("game_save", true).unwrap();
        assert_eq!(new_store.get_next_record_id(), 1);
        assert_eq!(new_store.get_num_records(), 0);
    }

    #[test]
    fn test_rms_failed_operations_do_not_mutate_state() {
        let mut store = RecordStore::new("test");
        let id = store.add_record(b"valid");
        assert_eq!(store.get_num_records(), 1);
        let orig_version = store.version;

        // Invalid record ID access/delete/set
        assert_eq!(store.get_record(999), None);
        assert!(!store.delete_record(999));
        assert!(!store.set_record(999, b"data"));

        // State remains completely intact
        assert_eq!(store.get_num_records(), 1);
        assert_eq!(store.version, orig_version);
        assert_eq!(store.get_record(id), Some(&b"valid"[..]));
    }

    #[test]
    fn test_rms_serialization_roundtrip_preserves_next_id_and_records() {
        let mut rms = RmsManager::new();
        {
            let store = rms.open("profile", true).unwrap();
            store.add_record(b"slot1");
            let id2 = store.add_record(b"slot2");
            store.delete_record(id2);
            assert_eq!(store.get_next_record_id(), 3);
        }

        // Serialize
        let bytes = rms.serialize();
        assert!(!bytes.is_empty());

        // Deserialize into fresh manager
        let mut rms2 = RmsManager::new();
        assert!(rms2.deserialize(&bytes).is_ok());

        let restored = rms2.open("profile", false).unwrap();
        assert_eq!(restored.get_num_records(), 1);
        assert_eq!(
            restored.get_next_record_id(),
            3,
            "next_id must be restored across sessions"
        );
        assert_eq!(restored.get_record(1), Some(&b"slot1"[..]));
    }
}
