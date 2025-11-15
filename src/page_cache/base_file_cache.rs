

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