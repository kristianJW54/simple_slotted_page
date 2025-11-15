

//NOTE: BaseFileCache is the source gateway for page handling between in-memory and on-disk

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use crate::page::PageID;

pub(crate) struct BaseFileCache {

	// TODO Replace the VedDeque with LRU Cache Linear Hashing?

	// cache states
	// write buffer
	write_buffer: Arc<Mutex<HashMap<PageID,Arc<[u8]>>>>,
	// read cache
	read_cache: RwLock<HashMap<PageID,Arc<[u8]>>>,

}

//NOTE: A PageRef is shared in read cache - if we take a page into write cache and remove from read cache there may still be active
// transaction references on that page
// when we want to write to the page, we try to get_mut() and if we fail we must rebuild the page
// when readers try to read a page that isn't in the read cache it must load from disk
// as soon as committed pages are flushed the route page is updated to point to the newer pages and
// subsequent readers follow the new chain and will load those pages from disk skipping older versions