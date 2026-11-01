
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

        let mut ids: Vec<i32> = store.records.keys().copied().collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![a, b, c]);

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
        let ids: Vec<i32> = store.records.keys().copied().collect();
        assert_eq!(ids, vec![b]);
    }
}
