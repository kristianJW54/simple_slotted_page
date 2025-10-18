# Slotted Pages

---

> Slotted pages are used by B+tree's as an efficient means of organizing stored blocks of memory on disk

The structure of a page has three main elements:

1. Header
2. Slot Array
3. Data Cells

In-between the Slot Array and Data Cells exists free space

| Header | Slot Array => | -------- Free Space -------- | <= Data Cells |
| ------ | ------------- | ---------------------------- | ------------- |

Cells are appended from the back of the array while Slots/Pointers are added to the Slot Array after the header.