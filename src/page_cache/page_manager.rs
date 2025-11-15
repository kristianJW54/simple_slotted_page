

// TODO Think about the different high level transaction pages we need which will be used by low-level builder pages

//NOTE: MVCC page mutation model
// -
// Every page is first accessed through an immutable snapshot (PageImpl:
// Arc<[u8]>). Writers never mutate committed pages. On insert we follow
// one of three paths:
// -
// 1. In-place mutation (fast path):
//    Allowed only if the page is uncommitted (it already lives exclusively
//    in the transaction’s write buffer) and there is enough free space to
//    insert without changing the page layout. In this case we call
//    get_page_mut() to obtain PageMut, which writes into the transaction-
//    local mutable Vec<u8>.
// -
// 2. Rebuild without split:
//    If the page is committed, or lacks room for in-place mutation,
//    we build a brand new leaf page. All existing key/value pairs are
//    copied into a new page, the new entry is added, and this new page
//    replaces the old one for the writer. The old page remains visible
//    to readers (MVCC).
// -
// 3. Rebuild with split:
//    If the page overflows after the insert, we build two new pages
//    (left and right) and return a split key to the parent. The old page
//    again stays untouched for readers.
// -
// Summary:
// - PageImpl is always immutable (MVCC snapshot).
// - PageMut exists only for uncommitted pages in the write buffer.
// - Committed pages are never mutated; they are rebuilt instead.
// - This guarantees snapshot isolation and safe copy-on-write behavior.