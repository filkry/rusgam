use std::cell::{RefCell};
use std::collections::HashMap;
use std::collections::hash_map::{DefaultHasher};
use std::hash::{Hash, Hasher};

use allocate::{SAllocator, SAllocatorRef, SLinearAllocator, SYSTEM_ALLOCATOR};
use collections::{SVec};

pub struct SEntry {
    _bytes: SVec<u8>,
    self_ptr: *const str,
}

#[derive(Debug, Copy, Clone)]
pub struct SHashedStr {
    uid: u64,
    _debug_ptr: *const str,
}

pub struct SDB {
    allocator: SAllocatorRef,
    entries: HashMap::<u64, SEntry>, // $$$FRK(TODO): need own hashmap
                                     // $$$FRK(TODO): default hasher is NON DETERMINISTIC ACROSS threads!
}

pub struct SThreadDB {
    _allocator: SAllocator,
    db: SDB,
}

impl SEntry {
    pub fn new(source: &str, allocator: &SAllocatorRef) -> Result<Self, &'static str> {
        let mut bytes = SVec::new(allocator, source.len() + 1, 0)?;
        for byte in source.bytes() {
            bytes.push(byte);
        }

        unsafe {
            let self_ptr = std::str::from_utf8_unchecked(bytes.as_ref()) as *const str;

            Ok(Self {
                _bytes: bytes,
                self_ptr,
            })
        }
    }
}

impl PartialEq for SHashedStr {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl SDB {
    pub fn new(allocator: SAllocatorRef) -> Self {
        Self {
            allocator: allocator,
            entries: HashMap::new(),
        }
    }

    pub fn hash_str(&mut self, source: &str) -> SHashedStr {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        let uid = hasher.finish();

        if let Some(entry) = self.entries.get(&uid) {
            SHashedStr{
                uid,
                _debug_ptr: entry.self_ptr,
            }
        }
        else {
            let new_entry = SEntry::new(source, &self.allocator).expect("string DB out of memory");
            let result = SHashedStr{
                uid,
                _debug_ptr: new_entry.self_ptr,
            };
            self.entries.insert(uid, new_entry);
            result
        }
    }
}

impl SThreadDB {
    pub fn new(parent_allocator: SAllocatorRef, allocator_bytes: usize) -> Self {
        let linear_allocator_int = SLinearAllocator::new(parent_allocator.clone(), allocator_bytes, 8).expect("couldn't allocate thread string db");
        let linear_allocator = SAllocator::new(linear_allocator_int);
        let linear_allocator_ref = linear_allocator.as_ref();

        Self {
            _allocator: linear_allocator,
            db: SDB::new(linear_allocator_ref),
        }
    }
}

impl Drop for SThreadDB {
    fn drop(&mut self) {
        // -- must drain the hashmap first because it contains all the references to our linear allocator
        self.db.entries.drain();
    }
}

thread_local! {
    pub static THREAD_STRING_DB : RefCell::<SThreadDB> =
        RefCell::new(SThreadDB::new(SYSTEM_ALLOCATOR(), 8 * 1024 * 1024));
}

pub fn hash_str(source: &str) -> SHashedStr {
    THREAD_STRING_DB.with(|db| {
        db.borrow_mut().db.hash_str(source)
    })
}

#[test]
fn test_simple() {
    let hello_world1 = hash_str("hello_world");
    let hello_world2 = hash_str("hello_world");
    let diff_hash = hash_str("different");

    assert_eq!(hello_world1.uid, hello_world2.uid);
    assert_eq!(hello_world1._debug_ptr, hello_world2._debug_ptr);
    assert!(hello_world1.uid != diff_hash.uid);
    assert!(hello_world1._debug_ptr != diff_hash._debug_ptr);
}
