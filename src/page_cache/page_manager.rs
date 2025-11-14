

//NOTE: It is the Page Managers job to manage the pages from disk into buffer and then handing out pointers
// to pages from the buffer.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use crate::raw_page::slotted_page::{PageID, RawPage, RawPageType};

pub type PageBuffer = Arc<Mutex<PageCache>>;

pub(crate) struct PageCache {
    // TODO Would need a sharded partition hash map to avoid lock bottleneck
    page_table: HashMap<u16, u16>,
    cache: Vec<Arc<PageFrame>>,
}

impl PageCache {
    pub fn new() -> Self {
        Self { page_table: HashMap::new(), cache: Vec::new(), }
    }

    pub fn load_page(&mut self, page: RawPage) -> Result<(), String> {
        // TODO Need to return a PageReadGuard
        Ok(())
    }
}

pub fn new_buffer() -> PageBuffer {
    Arc::new(Mutex::new(PageCache::new()))
}

pub(crate) struct PageFrame {
    id: u16,
    page_type: u8,
    pin_count: AtomicUsize,
    dirty: AtomicBool,
    inner_page: RawPage,
    lock: RwLock<()>,
}

// TODO Need to think about an intermediary layer for PageHandle/PageAccess which can coordinate latch handling and intent



pub(crate) struct PageReadGuard {
    page: Arc<PageFrame>,
    // Do we want this to store transactional memory?
}

pub(crate) struct PageWriteGuard {
    page: Arc<PageFrame>,
}


#[test]
fn test_page_cache() {

    let cache = new_buffer();

    // This would be performed by the b-tree methods
    let mut c = cache.as_ref().lock().unwrap();

    // dummy load from db
    let disk_page = RawPage::new(PageID(32), RawPageType::Internal);

    // dummy cache from disk and request from b-tree/transaction
    let page = c.load_page(disk_page).unwrap();

    // Unlock the cache?

}

