
// Cells are the data blocks within the page
// They can be made up of fixed sized data and variable length data but not both


// Cells have different types/functions
// - Key Cells which are internal on the B+tree and point to leaf cells where values are stored
// - KeyValue Cells which are keys which the data associated with the key stored next to them

use std::marker::PhantomData;


// This will be returned by Page on methods that it uses to call into Cell
// So do we want to wrap the underlying Cell in this enum?
enum CellReturn<'page> {
    Inline(Cell<'page>), /* Inline(*Need something here*) */
    Overflow, /* Overflow(*Need something here*) */
}

#[repr(C)]
struct Cell<'page> {
    // fields
    _marker: PhantomData<&'page ()>,
}

struct Overflow<'page> {
    _marker: PhantomData<&'page ()>,
    // Returned bytes of current cell value
    // Owned fields such as ID and total len to follow the chained value in overflow
}

