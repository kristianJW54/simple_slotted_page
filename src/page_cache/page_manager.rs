

//NOTE: It is the Page Managers job to manage the pages from disk into buffer and then handing out pointers
// to pages from the buffer.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use crate::raw_page::slotted_page::RawPage;

pub(crate) struct PageCache {
    // TODO Would need a sharded partition hash map to avoid lock bottleneck
    page_table: HashMap<u16, u16>,
    cache: Vec<Arc<PageFrame>>,
    cache_lock: RwLock<()>, // Really rough for now
}


enum PageLatchMode {
    Shared,
    WriteIntent,
    Exclusive,
}

struct Latch {
    shared_count: AtomicUsize,
    intent_count: AtomicUsize,
    exclusive_count: AtomicUsize,
}

pub(crate) struct PageFrame {
    id: u16,
    page_type: u8,
    pin_count: AtomicUsize,
    dirty: AtomicBool,
    inner_page: RawPage,
    lock: RwLock<()>,
}

// TODO Need to think about an intermeddiery layer for PageHandle/PageAccess which can coordinate latch handling and intent

pub(crate) struct PageReadGuard {
    page: Arc<PageFrame>,
}

pub(crate) struct PageWriteGuard {
    page: Arc<PageFrame>,
}

