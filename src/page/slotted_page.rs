

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

use std::{mem, slice};

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
const PAGE_TYPE_SIZE: usize = 1;
const PAGE_TYPE_OFFSET: usize = PAGE_HEADER_FLAG_OFFSET + PAGE_HEADER_FLAG_SIZE; // 9
const PAGE_SLOT_COUNT_SIZE: usize = 1;
const PAGE_SLOT_COUNT_OFFSET: usize = PAGE_TYPE_OFFSET + PAGE_TYPE_SIZE; // 10
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

enum PageType {
	Internal,
	Leaf,
}

impl TryFrom<u8> for PageType {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(PageType::Internal),
			2 => Ok(PageType::Leaf),
			_ => Err(()),
		}
	}
}

impl From<PageType> for u8 {
	fn from(value: PageType) -> Self {
		match value {
			PageType::Internal => 1,
			PageType::Leaf => 2,
		}
	}
}

pub struct Page {
	slotted_page: [u8; PAGE_SIZE],
}

// Start with header (which we may move internal methods to a header.rs module)

impl Page {
	// TODO: Add a new_with_data?
	fn new(page_id: PageID, page_type: PageType) -> Self {
		let mut slotted_page = [0u8; PAGE_SIZE];
		let size = slotted_page.len();
		// Add the id at the beginning of the header
		slotted_page[PAGE_HEADER_ID_OFFSET..PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE]
			.copy_from_slice(&page_id.0.to_le_bytes());
		// Add the flags
		slotted_page[PAGE_HEADER_FLAG_OFFSET..PAGE_HEADER_FLAG_OFFSET + PAGE_HEADER_FLAG_SIZE]
			.copy_from_slice(&0u16.to_le_bytes());
		// Write the page type
		slotted_page[PAGE_TYPE_OFFSET..PAGE_TYPE_OFFSET + PAGE_TYPE_SIZE]
			.copy_from_slice(&u8::from(page_type).to_le_bytes());
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

	// TODO This would have to be with an index

	fn insert_slot(&mut self, slot: SlotEntry) -> Result<(), String> {

		let slot_bytes = slot.to_bytes();

		let free_start = self.free_start();
		let free_end = self.free_end();

		if (free_end - free_start) < 4 {
			return Err(format!("Not enough free space {}", (free_end - free_start)));
		}

		self.slotted_page[free_start..free_start + mem::size_of::<SlotEntry>()]
			.copy_from_slice(slot_bytes);

		let header = self.header_mut();
		header.slot_count += 1;
		header.free_start += mem::size_of::<SlotEntry>() as u16;

		Ok(())
	}

	// NOTE: Need to check for NonNull ptr OR maybe wrap ptr in NonNull etc

	// header returns a reference to a HeaderStruct not exclusive or mutable
	fn header_mut(&mut self) -> &mut Header {
		assert_eq!(mem::size_of::<Header>(), HEADER_SIZE, "Header size not equal to header struct alignment");
		// SAFETY: We guarantee that:
		//  - PAGE_SIZE ≥ HEADER_OFFSET + size_of::<HeaderStruct>()
		//  - slotted_page.as_ptr().add(HEADER_OFFSET) is properly aligned for HeaderStruct
		//  - the bytes at that offset have been initialized to HeaderStruct form
		unsafe {
			//SAFETY: We are safe to return a mutable ref to HeaderStruct because borrow checker
			// enforces that only one exclusive of page is available and, therefore, we cannot create
			// more than one mut ref HeaderStruct

			// Here we are saying we want a mut ref of the thing that the pointer is pointing to
			&mut *(self.slotted_page.as_ptr().add(PAGE_OFFSET) as *mut Header)
		}
	}

	// Header specific reference methods
	const fn slot_count(&self) -> u8 {
		self.slotted_page[PAGE_SLOT_COUNT_OFFSET]
	}

	fn free_start(&self) -> usize {
		let bytes = &self.slotted_page[HEADER_FREE_START_OFFSET.. HEADER_FREE_START_OFFSET + HEADER_FREE_LOCATOR_SIZE];
		u16::from_le_bytes(bytes.try_into().unwrap()) as usize
	}

	fn free_end(&self) -> usize {
		let bytes = &self.slotted_page[HEADER_FREE_END_OFFSET.. HEADER_FREE_END_OFFSET + HEADER_FREE_LOCATOR_SIZE];
		u16::from_le_bytes(bytes.try_into().unwrap()) as usize
	}

	fn slot_dir_mut(&mut self) -> SlotDir<'_> {
		let header = HEADER_SIZE;
		let lower = self.free_start();
		let sc = self.slot_count() as usize;
		let slot_bytes = &mut self.slotted_page[header..lower];

		SlotDir::from_raw_bytes_mut(sc, slot_bytes)

	}

	// TODO Need a key_lookup

	// Page main methods
	// pub fn get(&self, slot_id: SlotID) -> Option<&[u8]> {}
	// pub fn insert(&mut self, record: &[u8]) -> Result<SlotID, String> {}
	// pub fn remove(&mut self, slot_id: SlotID) -> Result<(), String> {}

}

#[repr(C)]
#[derive(Debug)]
struct Header {
	page_id:    u64,
	flags:      u16,
	page_type:  u8,
	slot_count: u8,
	free_start: u16,
	free_end:   u16,
	// Final Offset = 16 (after padding) = multiple of 8 so no further padding
}


// The only way to index into cells within the page is through the SlotDir which is the directory
// for the ptr and len of all cells

//NOTE: We do so in memory using Little-Endian and when we flush to disk we enforce that byte layout by writing it that way
// To enforce cross architecture portability

struct SlotDir<'a> {
	se: &'a mut [SlotEntry],
}

//TODO
// - SlotDir needs to implement Iter
// - Need to have a method for key comparison
// - Simple iteration with entry comparison to find index
// - If we assume that Page creates space for a new slot already - then we can house insertion logic here
impl SlotDir<'_> {
	fn from_raw_bytes_mut(slot_count: usize, bytes: &'_ mut [u8]) -> SlotDir<'_> {

		let expected_len = slot_count * mem::size_of::<SlotEntry>();
		let actual_len = bytes.len();

		assert_eq!(
			actual_len,
			expected_len,
			"SlotDir region length mismatch: got {} bytes, expected {} bytes ({} entries * {} bytes each)",
			actual_len,
			expected_len,
			slot_count,
			mem::size_of::<SlotEntry>(),
		);

		// Sanity check: Slot Entry must be of size 2
		assert_eq!(mem::size_of::<SlotEntry>(), 2);

		//SAFETY: We ensure the correct alignment and size of SlotEntry and that the bytes point to a valid
		// region of data as this method is called only within the Page Impl block and the byte slice extracted
		// is the exact region of the slot array.
		// We also ensure that is only called in an exclusive mutably borrowed method ties to the lifetime of the Page
		let slot_entries = unsafe {
			slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut SlotEntry, slot_count)
		};
		SlotDir { se: slot_entries }
	}
}

//TODO Impl SlotDir and also implement Iter to go through SlotEntries
// The Impl block should define methods for extracting SlotEntry and working with them?

// We need to be able to cast to SlotEntry to avoid from_le_bytes.
#[repr(C)]
#[derive(Debug)]
struct SlotEntry {
	offset: u8,
	len: u8,
}

impl SlotEntry {

	fn to_bytes(&self) -> &[u8] {
		unsafe {
			slice::from_raw_parts((self as *const SlotEntry).cast::<u8>(), mem::size_of::<SlotEntry>())
		}
	}

}


// We would want to use the impl block to build cell views but tie them to the lifetime of the page


#[cfg(test)]
mod tests {
	use std::mem;
	use super::*;

	#[test]
	fn test_new_page() {

		let page_id = PageID(1234u64);
		let page = Page::new(page_id, PageType::Internal);

		let mut id = [0u8; PAGE_HEADER_ID_SIZE];
		id.copy_from_slice(&page.slotted_page[PAGE_HEADER_ID_OFFSET .. PAGE_HEADER_ID_OFFSET + PAGE_HEADER_ID_SIZE]);
		let want_id = u64::from_le_bytes(id);

		// PageID implements clone so we can access it here as well
		assert_eq!(page_id.0, want_id);

	}

	#[test]
	fn test_unsafe_header() {

		let page_id = PageID(1234u64);
		let mut page = Page::new(page_id, PageType::Internal);

		let header = page.header_mut();

		println!("{:?}", header);

		println!("size_of::<HeaderStruct>() = {}", mem::size_of::<Header>());
		println!("align_of::<HeaderStruct>()  = {}", mem::align_of::<Header>());

	}

	#[test]
	fn zero_slot_dir_from_new_page() {
		let page_id = PageID(1234u64);
		let mut page = Page::new(page_id, PageType::Internal);

		let slot_dir = page.slot_dir_mut();

		println!("slot_dir {:?}", slot_dir.se);

	}

	#[test]
	fn slot_dir_after_adding_slot() {

		let page_id = PageID(1234u64);
		let mut page = Page::new(page_id, PageType::Internal);

		let slot_dir = page.slot_dir_mut();

		assert_eq!(slot_dir.se.len(), 0);

		drop(slot_dir);

		let se = SlotEntry { offset: 1, len: 10 };

		page.insert_slot(se).unwrap();

		let slot_dir = page.slot_dir_mut();

		println!("se {:?}", slot_dir.se[0]);

	}
}

















