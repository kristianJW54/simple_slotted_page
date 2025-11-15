
// First we need a global meta page which will be the entry point for all transactions

use crate::page::PageID;

pub(crate) struct MetaPageSwitch {
	active_slot: u8,
	meta_slot: [MetaPage; 2],
}

pub(crate) struct MetaPage {
	version: u64, // Change to new-type
	system_catalogue_head: PageID, // For now this will be a single b-tree header route
	free_list_head: PageID,

	checksum: u64,

}


pub(crate) struct Page {
	
}