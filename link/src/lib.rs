mod allocator;

/// WebAssembly export that allocates a pointer (linear memory offset) that can
/// be used for a string.
///
/// This is an ownership transfer, which means the caller must call
/// [`deallocate`] when finished.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "allocate"))]
pub extern "C" fn _allocate(size: u32) -> *mut u8 {
    allocator::allocate(size as usize)
}

/// WebAssembly export that deallocates a pointer of the given size (linear
/// memory offset, byteCount) allocated by [`allocate`].
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "deallocate"))]
pub unsafe extern "C" fn _deallocate(ptr: u32, size: u32) {
    unsafe { allocator::deallocate(ptr as *mut u8, size as usize) };
}

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    /// WebAssembly import which prints a string (linear memory offset,
    /// byteCount) to the console.
    ///
    /// Note: This is not an ownership transfer: Rust still owns the pointer
    /// and ensures it isn't deallocated during this call.
    #[link_name = "log"]
    fn _log(ptr: u32, size: u32);
}

fn log(s: &str) {
    let (ptr, size) = unsafe { allocator::string_to_ptr(s) };
    unsafe { _log(ptr, size) }
}

fn on_create() {
    log("on_create called");
}

/// The main function for the `on_create` event.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "on_create"))]
pub unsafe extern "C" fn _on_create(_ptr: u32, _len: u32) {
    on_create();
}
