extern crate alloc;
extern crate core;
extern crate wee_alloc;

use alloc::vec::Vec;
use std::mem::MaybeUninit;

/// Set the global allocator to the WebAssembly optimized one.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Allocates size bytes and leaks the pointer where they start.
pub fn allocate(size: usize) -> *mut u8 {
    // Allocate the amount of bytes needed.
    let vec: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); size];

    // into_raw leaks the memory to the caller.
    Box::into_raw(vec.into_boxed_slice()) as *mut u8
}

/// Retakes the pointer which allows its memory to be freed.
pub(crate) unsafe fn deallocate(ptr: *mut u8, size: usize) {
    let _ = unsafe { Vec::from_raw_parts(ptr, 0, size) };
}

/// Returns a string from WebAssembly compatible numeric types representing
/// its pointer and length.
pub unsafe fn ptr_to_string(ptr: u32, len: u32) -> String {
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, len as usize) };
    let utf8 = unsafe { std::str::from_utf8_unchecked_mut(slice) };
    utf8.to_owned()
}
/// Returns a pointer and size pair for the given string in a way compatible
/// with WebAssembly numeric types.
///
/// Note: This doesn't change the ownership of the String. To intentionally
/// leak it, use [`std::mem::forget`] on the input after calling this.
pub unsafe fn string_to_ptr(s: &str) -> (u32, u32) {
    (s.as_ptr() as u32, s.len() as u32)
}

/// WebAssembly export that allocates a pointer (linear memory offset) that can
/// be used for a string.
///
/// This is an ownership transfer, which means the caller must call
/// [`deallocate`] when finished.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "allocate"))]
pub extern "C" fn _allocate(size: u32) -> *mut u8 {
    allocate(size as usize)
}

/// WebAssembly export that deallocates a pointer of the given size (linear
/// memory offset, byteCount) allocated by [`allocate`].
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "deallocate"))]
pub unsafe extern "C" fn _deallocate(ptr: u32, size: u32) {
    unsafe { deallocate(ptr as *mut u8, size as usize) };
}

/// Reads a pointer and length from an encoded u64 value.
/// Format: the upper 32 bits represent the pointer, and the lower 32 bits represent the length
pub fn read_ptr_len(encoded: u64) -> (u32, u32) {
    let ptr = (encoded >> 32) as u32;
    let len = (encoded & 0xFFFFFFFF) as u32;
    (ptr, len)
}
