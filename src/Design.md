
### Layers

**Base**

1. At the base interaction with the Disk is the **BaseFileCache**

This holds:

- Read LRU Cache
- Write LRU Cache
- Meta & Metrics

2. Second is the **TransactionView**

It holds a shared view into the BaseFileCache for the duration of a transaction.
This is what the b-tree talks to when it calls into the cache

3. **PageMut** used by the b-tree

- A get_mut_page() -> PageMut
    - This calls into the base cache and into the write LRU Cache to fetch and clone the page
    - The PageMut holds a WriteAblePage which owns a shared reference to the WriteCache in the BaseFileCache
    - PageMut also stores TransactionView, although page writes only happen through WriteablePage to avoid the slow
      TransactionView being used on the hot path leaving the TransactionView to handle book-keeping.
