

// Slotted pages have 3 main elements
// 1. Header
// 2. Pointers
// 3. Cells

// Then in between pointers and cells there is free space

// Each page must have a UID to be located
// A DBMS uses an indirection layer to map page ID's to physical locations

//---- Page Types ------//
// - Hardware -> Usually 4kb (the largest block of memory which the storage device can guarantee safe writes)
// - OS Page  -> Usually 8kb
// - Database -> Usually 16kb

/*NOTE:
	---- Page Header ------
	Usually 24 bytes
	Contains meta-data about the pages contents

	The header keeps track of:
	- Log sequence number (for crash recovery)
	- Checksum
	- Flags
	- Number of slots used
	- Offset of the starting location of the last slot used (end of free space)
	- Offset to the ending location of the slot array (start of free space)
	- PageVersion number

	---- Slotted Array -----
	Usually 4 bytes per item

	- Array Item is a tuple (offset, length)

 */

//NOTE: In order to implement the slotted page we will need to work with a contiguous buffer as structs
// add padding and alignment

const PAGE_SIZE: usize = 4096;

// Page flags bit arrays
const TUPLE_FLAG: u16 = 1 << 0;

// Page ID new_type
pub struct PageID(pub u64);
pub struct SlotID(pub u16);
pub struct RowID {
	p: PageID,
	s: SlotID,
}
// Free Space Locators
pub struct LocationIndex(u16);

pub struct Page {
	slotted_page: [u8; PAGE_SIZE],
}

// Start with header (which we may move internal methods to a header.rs module)

impl Page {

	// Header helpers here

	// For use with new
	// -
	// -
	// TODO: Add a new_with_data?
	fn new(page_id: PageID, page_type: u16) -> Self {
		// TODO : Add the header and structure to the slotted_page
		let mut slotted_page = [0u8; PAGE_SIZE];

		// Add the id at the beginning of the header
		let id: [u8; 8] = page_id.0.to_le_bytes();
		slotted_page[..8].copy_from_slice(&id);

		// Add the flags
		slotted_page[8.. 10].copy_from_slice(&page_type.to_le_bytes());

		// For free space locators we do nothing because they are zero

		Self { slotted_page, }
	}

	// Page main methods
	// pub fn get(&self, slot_id: SlotID) -> Option<&[u8]> {}
	// pub fn insert(&mut self, record: &[u8]) -> Result<SlotID, String> {}
	// pub fn remove(&mut self, slot_id: SlotID) -> Result<(), String> {}
	// pub fn compact(&mut self) -> Result<(), String> {}

	// Internal Helpers
	// fn get_header(&self) -> HeaderView {}
	// fn set_header(&mut self, header: HeaderView) {}
}

// NOTE: Here we define a header view for copying out the header data from the page

struct HeaderView {
	id: PageID,
	flags: u16,
	start_free_space: u16,
	end_free_space: u16,
	// More to come...
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_page() {

		let page_id = PageID(1234u64);
		let page_type: u8 = (1 << 2) as u8;

		println!("page_type: {:08b} -> {page_type}", page_type);

		let page = Page::new(page_id, TUPLE_FLAG);

		println!("page - {:?}", page.slotted_page);

	}

	#[test]
	fn to_le_bytes_test() {

		let v = 16u16;

		println!("v -> {v} -> {:?}", v.to_le_bytes())


	}
}

















