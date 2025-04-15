mod allocator;
pub mod entry_capnp {
    include!(concat!(env!("OUT_DIR"), "/entry_capnp.rs"));
}

macro_rules! capnp_str {
    ($expr:expr) => {
        $expr.map_err(Error::Capnp)?.to_str().map_err(Error::Utf8)?
    };
}

#[derive(Debug)]
enum Error {
    Capnp(capnp::Error),
    Utf8(std::str::Utf8Error),
}

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    #[link_name = "debug"]
    fn _debug(ptr: u32, size: u32);

    #[link_name = "error"]
    fn _error(ptr: u32, size: u32);
}

fn debug(s: &str) {
    let (ptr, size) = unsafe { allocator::string_to_ptr(s) };
    unsafe { _debug(ptr, size) }
}

fn error(s: &str) {
    let (ptr, size) = unsafe { allocator::string_to_ptr(s) };
    unsafe { _error(ptr, size) }
}

fn on_create(ptr: u32, len: u32) -> Result<(), Error> {
    debug("on_create called");

    // Read the Cap'n Proto message from the provided pointer and length.
    let buf: &[u8] = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let segments = &[buf];
    let message = capnp::message::Reader::new(
        capnp::message::SegmentArray::new(segments),
        core::default::Default::default(),
    );
    let entry = message
        .get_root::<entry_capnp::entry::Reader>()
        .map_err(Error::Capnp)?;

    let id = capnp_str!(entry.get_id());
    let name = capnp_str!(entry.get_name());

    debug(name);
    Ok(())
}

/// The main function for the `on_create` event.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "on_create"))]
pub unsafe extern "C" fn _on_create(ptr: u32, len: u32) {
    if let Err(e) = on_create(ptr, len) {
        error(&format!("Error in on_create: {:?}", e));
    }
}
