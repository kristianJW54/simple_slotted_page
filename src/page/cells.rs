
// Cells are the data blocks within the page
// They can be made up of fixed sized data and variable length data but not both


// Cells have different types/functions
// - Key Cells which are internal on the B+tree and point to leaf cells where values are stored
// - KeyValue Cells which are keys which the data associated with the key stored next to them


/*
** The content of a cell looks like this:
**
**    SIZE    DESCRIPTION
**      4     Page number of the left child. Omitted if leaf flag is set.
**     var    Number of bytes of data. Omitted if the zero data flag is set.
**     var    Number of bytes of key. Or the key itself if int key flag is set.
**      *     Payload
**      4     First page of the overflow chain.  Omitted if no overflow
**
** Overflow pages form a linked list.  Each page except the last is completely
** filled with data (page size - 4 bytes).  The last page can have as little
** as 1 byte of data.
**
**    SIZE    DESCRIPTION
**      4     Page number of next overflow page
**      *     Data
*/
