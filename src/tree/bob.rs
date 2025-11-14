

/*

A B-Tree is composed of 3 elements :

* The *BTreeHeader*, which contains the reference on the two other elements
* The *BTreeInfo*, which contains information relative to this **B-Tree** (this element is never updated)
* The *RootPage*, which is the root of the **B-Tree**

*/

// NOTE: The b-tree directory which gives a transaction it's starting point for a route page

// TODO Start here or at the transaction level