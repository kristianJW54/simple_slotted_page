

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

//NOTE: Pages are not compacted or rebuilt immediately
// Instead when a cell is removed - the pointer is nullified and we mark the space where the data occupied
// as free blocks - this allows new data being inserted to measure if it can fit in one of the free blocks
// and determine what block will allow for the least remaining space
// If total free space allows but the fragmentation does not then we can rebuilt the page - otherwise we must
// Use overflow page

use std::mem;

const PAGE_SIZE: usize = 4096;
const SLOT_ENTRY_SIZE: usize = 4;

// Page flags bit arrays
const TUPLE_FLAG: u16 = 1 << 0;

// Header layout constants
const PAGE_OFFSET: usize = 0;
const PAGE_HEADER_ID_SIZE: usize = 8;
const PAGE_HEADER_ID_OFFSET: usize = PAGE_OFFSET;  // 0
const PAGE_HEADER_FLAG_SIZE: usize = 2;
const PAGE_HEADER_FLAG_OFFSET: usize = PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE;  // 8
const PAGE_SLOT_COUNT_SIZE: usize = 2;
const PAGE_SLOT_COUNT_OFFSET: usize = PAGE_HEADER_FLAG_OFFSET + PAGE_HEADER_FLAG_SIZE; // 10
const HEADER_FREE_LOCATOR_SIZE: usize = 2; // free_start or free_end size
const HEADER_FREE_START_OFFSET: usize = PAGE_SLOT_COUNT_OFFSET + PAGE_SLOT_COUNT_SIZE; // 12
const HEADER_FREE_END_OFFSET: usize = HEADER_FREE_START_OFFSET + HEADER_FREE_LOCATOR_SIZE; // 14

const HEADER_SIZE: usize = HEADER_FREE_END_OFFSET + HEADER_FREE_LOCATOR_SIZE; // 16

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
	// TODO: Add a new_with_data?
	fn new(page_id: PageID, page_type: u16) -> Self {
		let mut slotted_page = [0u8; PAGE_SIZE];
		let size = slotted_page.len();
		// Add the id at the beginning of the header
		slotted_page[PAGE_HEADER_ID_OFFSET..PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE]
			.copy_from_slice(&page_id.0.to_le_bytes());
		// Add the flags
		slotted_page[PAGE_HEADER_FLAG_OFFSET..PAGE_HEADER_FLAG_OFFSET + PAGE_HEADER_FLAG_SIZE]
			.copy_from_slice(&page_type.to_le_bytes());
		// Write page slot count
		slotted_page[PAGE_SLOT_COUNT_OFFSET..PAGE_SLOT_COUNT_OFFSET + PAGE_SLOT_COUNT_SIZE]
			.copy_from_slice(&0u16.to_le_bytes());
		// For free space locators - the first free space is end of header and last free space is end of array
		// As we have no data right now.
		slotted_page[HEADER_FREE_START_OFFSET..HEADER_FREE_START_OFFSET + HEADER_FREE_LOCATOR_SIZE]
			.copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
		slotted_page[HEADER_FREE_END_OFFSET..HEADER_FREE_END_OFFSET + HEADER_FREE_LOCATOR_SIZE]
			.copy_from_slice(&(size as u16).to_le_bytes());

		Self { slotted_page, }
	}

	// header returns a reference to a HeaderStruct not exclusive or mutable
	fn header(&self) -> &HeaderStruct {
		assert_eq!(mem::size_of::<HeaderStruct>(), HEADER_SIZE, "Header size not equal to header struct alignment");
		// SAFETY: We guarantee that:
		//  - PAGE_SIZE ≥ HEADER_OFFSET + size_of::<HeaderStruct>()
		//  - buf.as_ptr().add(HEADER_OFFSET) is properly aligned for HeaderStruct
		//  - the bytes at that offset have been initialized to HeaderStruct form
		unsafe {
			// Need to understand this...
			&*(self.slotted_page.as_ptr().add(PAGE_OFFSET) as *const HeaderStruct)
		}
	}

	// Page main methods
	// pub fn get(&self, slot_id: SlotID) -> Option<&[u8]> {}
	// pub fn insert(&mut self, record: &[u8]) -> Result<SlotID, String> {}
	// pub fn remove(&mut self, slot_id: SlotID) -> Result<(), String> {}

}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct HeaderStruct {
	page_id:   u64, // Needs 8 bytes alignment -> Offset 0 = OK
	flags:     u16, // Needs 2 bytes alignment -> Offset 8 % 2 = 0 = OK
	slot_count: u8, // Needs 1 byte  alignment -> Offset 10 % 2 = 0 = OK
	free_start: u16,// Needs 2 bytes alignment -> Offset 12 % 2 = 0 = OK
	free_end:   u16,// Needs 2 bytes alignment -> Offset 12 % 2 = 0 = OK
	// Final Offset = 16 (after padding) = multiple of 8 so no further padding
}


#[cfg(test)]
mod tests {
	use std::mem;
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
	fn test_unsafe_header() {

		let page_id = PageID(1234u64);
		let page = Page::new(page_id, TUPLE_FLAG);

		let header = page.header();

		println!("{:?}", header);

		let test_head = HeaderStruct{page_id: 0, flags: 1, slot_count: 2, free_start: 3, free_end: 4};
		// let risky = *(&test_head); // Copy fresh struct as HeaderStruct implements Copy
		let risky = unsafe { *(&test_head as *const HeaderStruct) }; // Zero copy uses raw pointers
		println!("Risky -> {:?}", risky);

		println!("size_of::<HeaderStruct>() = {}", mem::size_of::<HeaderStruct>());
		println!("align_of::<HeaderStruct>()  = {}", mem::align_of::<HeaderStruct>());

	}

	#[test]
	fn to_le_bytes_test() {

		let v = 16u16;

		println!("v -> {v} -> {:?}", v.to_le_bytes())


	}
}

















