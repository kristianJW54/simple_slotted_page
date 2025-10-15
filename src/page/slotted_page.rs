

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

// Header layout constants
const PAGE_OFFSET: usize = 0;
const PAGE_HEADER_ID_SIZE: usize = 8;
const PAGE_HEADER_ID_OFFSET: usize = PAGE_OFFSET;  // 0
const PAGE_HEADER_FLAG_SIZE: usize = 2;
const PAGE_HEADER_FLAG_OFFSET: usize = PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE;  // 8
const PAGE_SLOT_COUNT_SIZE: usize = 1;
const PAGE_SLOT_COUNT_OFFSET: usize = PAGE_HEADER_FLAG_OFFSET + PAGE_HEADER_FLAG_SIZE; // 9
const HEADER_FREE_LOCATOR_SIZE: usize = 2; // free_start or free_end size
const HEADER_FREE_START_OFFSET: usize = PAGE_SLOT_COUNT_OFFSET + PAGE_SLOT_COUNT_SIZE; // 11
const HEADER_FREE_END_OFFSET: usize = HEADER_FREE_START_OFFSET + HEADER_FREE_LOCATOR_SIZE; // 13

const HEADER_SIZE: usize = HEADER_FREE_END_OFFSET + HEADER_FREE_LOCATOR_SIZE; // 15

// Page ID new_type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
		let mut slotted_page = [0u8; PAGE_SIZE];
		let size = slotted_page.len();
		// Add the id at the beginning of the header
		slotted_page[PAGE_HEADER_ID_OFFSET .. PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE]
			.copy_from_slice(&page_id.0.to_le_bytes());
		// Add the flags
		slotted_page[PAGE_HEADER_FLAG_OFFSET.. PAGE_HEADER_FLAG_OFFSET + PAGE_HEADER_FLAG_SIZE]
			.copy_from_slice(&page_type.to_le_bytes());
		// Write page slot count
		slotted_page[PAGE_SLOT_COUNT_OFFSET..PAGE_SLOT_COUNT_OFFSET + PAGE_SLOT_COUNT_SIZE]
			.copy_from_slice(&0u8.to_le_bytes());
		// For free space locators - the first free space is end of header and last free space is end of array
		// As we have no data right now.
		slotted_page[HEADER_FREE_START_OFFSET..HEADER_FREE_START_OFFSET + HEADER_FREE_LOCATOR_SIZE]
			.copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
		slotted_page[HEADER_FREE_END_OFFSET..HEADER_FREE_END_OFFSET + HEADER_FREE_LOCATOR_SIZE]
			.copy_from_slice(&(size as u16).to_le_bytes());

		Self { slotted_page, }
	}

	// Page main methods
	// pub fn get(&self, slot_id: SlotID) -> Option<&[u8]> {}
	// pub fn insert(&mut self, record: &[u8]) -> Result<SlotID, String> {}
	// pub fn remove(&mut self, slot_id: SlotID) -> Result<(), String> {}
	// pub fn compact(&mut self) -> Result<(), String> {}

	// Internal Helpers
	//OPTIMISATION -----
	//TODO : Get header view is copy - if this is a critical method consider exploring
	// raw pointer to get zero-copy view ...
	fn get_header_view(&self) -> HeaderView {
		let mut id = [PAGE_OFFSET as u8; PAGE_HEADER_ID_SIZE];
		id.copy_from_slice(&self.slotted_page[PAGE_HEADER_ID_OFFSET..PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE]);
		let page_id = u64::from_le_bytes(id);

		let mut flags= [PAGE_HEADER_FLAG_OFFSET as u8; PAGE_HEADER_FLAG_SIZE];
		flags.copy_from_slice(&self.slotted_page[PAGE_HEADER_FLAG_OFFSET .. PAGE_HEADER_FLAG_OFFSET + PAGE_HEADER_FLAG_SIZE]);
		let flag_field = u16::from_le_bytes(flags);

		let mut s_count = [PAGE_SLOT_COUNT_OFFSET as u8; PAGE_SLOT_COUNT_SIZE];
		s_count.copy_from_slice(&self.slotted_page[PAGE_SLOT_COUNT_OFFSET .. PAGE_SLOT_COUNT_OFFSET + PAGE_SLOT_COUNT_SIZE]);
		let slot_count_byte = u8::from_le_bytes(s_count);

		let mut start_slot = [HEADER_FREE_START_OFFSET as u8; HEADER_FREE_LOCATOR_SIZE];
		start_slot.copy_from_slice(&self.slotted_page[HEADER_FREE_START_OFFSET..HEADER_FREE_START_OFFSET + HEADER_FREE_LOCATOR_SIZE]);
		let start = u16::from_le_bytes(start_slot);

		let mut end_slot = [HEADER_FREE_END_OFFSET as u8; HEADER_FREE_LOCATOR_SIZE];
		end_slot.copy_from_slice(&self.slotted_page[HEADER_FREE_END_OFFSET..HEADER_FREE_END_OFFSET + HEADER_FREE_LOCATOR_SIZE]);
		let end = u16::from_le_bytes(end_slot);

		HeaderView {
			id: page_id,
			flags: flag_field,
			slot_count: slot_count_byte,
			start_free_space: start,
			end_free_space: end,
		}
	}
	// fn set_header(&mut self, header: HeaderView) {}
}

// NOTE: Here we define a header view for copying out the header data from the page

struct HeaderView {
	id: u64,
	flags: u16,
	slot_count: u8,
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
		let page = Page::new(page_id, TUPLE_FLAG);

		let mut id = [0u8; PAGE_HEADER_ID_SIZE];
		id.copy_from_slice(&page.slotted_page[PAGE_HEADER_ID_OFFSET .. PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE]);
		let want_id = u64::from_le_bytes(id);

		// PageID implements clone so we can access it here as well
		assert_eq!(page_id.0, want_id);

	}

	#[test]
	fn test_header_view() {
		// TODO : Implement header test
	}

	#[test]
	fn to_le_bytes_test() {

		let v = 16u16;

		println!("v -> {v} -> {:?}", v.to_le_bytes())


	}
}

















