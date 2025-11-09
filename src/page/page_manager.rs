use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use crate::page::slotted_page::Page;


// TODO Think about and make clean - Maybe a PageInner? Which has the Arc<Page>?
struct Pager {
	pages: RwLock<HashMap<u32, Arc<Page>>>,
}

/*
Pager is responsible for the loading and lifetimes and references of pages

For overflow pages we would simply load them as they are local to a particular page?
For traversal, we want to be careful about reference counts and pinning pages in memory
We may want to use implementation of drop with Arc references counting to control the pinning of pages

*/