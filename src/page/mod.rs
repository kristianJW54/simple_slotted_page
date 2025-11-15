pub mod slotted_page;
mod cells;
mod page;


// Page ID new_type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PageID(pub u64);