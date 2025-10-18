
// Cells are the data blocks within the page
// They can be made up of fixed sized data and variable length data but not both


// Cells have different types/functions
// - Key Cells which are internal on the B+tree and point to leaf cells where values are stored
// - KeyValue Cells which are keys which the data associated with the key stored next to them