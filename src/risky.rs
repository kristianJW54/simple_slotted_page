
// Risky unsafe testing area

// Function for splitting a mutable i32 slice

// TODO: Fix this - need to use unsafe to byass the borrow checker and make our own assertions

use std::slice;

fn split_slice(slice: &mut [i32], index: i32) -> Option<(&mut [i32], &mut [i32])> {

    let size = slice.len();
    assert!(index < size as i32);
    assert!(!slice.is_empty());

    // SAFETY: Take the mutable pointer to the slice (will only be unsage if we dereferene the pointer to mutate)
    let ptr = slice.as_mut_ptr();

    // NOTE: We cannot do this because the borrow checker argues that only one mutable reference should be active
    // let one = &mut slice[..index as usize];
    // let two = &mut slice[index as usize..];

    unsafe {
        //SAFETY: We assert that the index is less than the length of slice
        // We maintain that slice is not empty and contains data to not give Null pointer
        // Slice is i32 and aligned
        // Slice is not referenced by anything else as we pass in an exclusive reference to the function
        let one = slice::from_raw_parts_mut(ptr, index as usize);
        let two = slice::from_raw_parts_mut(ptr.add(index as usize), size - index as usize);
        Some((one, two))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_slice_test() {

        let mut v = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let refv = &mut v;
        if let Some((left, right)) = split_slice(refv, 5) {
            println!("left: {:?}, right: {:?}", left, right);
            // Can modify left and right within this scope
        }
        // mutable ref to refv is dropped after this scope

        println!("v: {:?}", v);

    }
}