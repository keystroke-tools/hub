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
    fn _debug(ptr: u32, size: u32) -> u64;

    #[link_name = "error"]
    fn _error(ptr: u32, size: u32) -> u64;

    #[link_name = "chunk_with_overlap"]
    fn _chunk_with_overlap(ptr: u32, size: u32) -> u64;

    #[link_name = "chunk_by_sentence"]
    fn _chunk_by_sentence(ptr: u32, size: u32) -> u64;
}

fn debug(s: &str) {
    let (ptr, size) = unsafe { allocator::string_to_ptr(s) };
    unsafe { _debug(ptr, size) };
}

fn error(s: &str) {
    let (ptr, size) = unsafe { allocator::string_to_ptr(s) };
    unsafe { _error(ptr, size) };
}

macro_rules! read_chunk_result {
    ($ptr:expr, $size:expr) => {{
        let slice = unsafe { core::slice::from_raw_parts($ptr as *const u8, $size as usize) };
        let mut cursor = std::io::Cursor::new(slice);
        let message =
            capnp::serialize::read_message(&mut cursor, capnp::message::ReaderOptions::new())
                .map_err(Error::Capnp)?;

        let chunk_result = message
            .get_root::<entry_capnp::chunk_result::Reader>()
            .map_err(Error::Capnp)?;

        let chunks_reader = chunk_result.get_chunks().map_err(Error::Capnp)?;

        let mut chunks = Vec::new();
        for i in 0..chunks_reader.len() {
            let chunk = capnp_str!(chunks_reader.get(i));
            chunks.push(chunk.to_string());
        }

        Ok(chunks)
    }};
}

/// Chunks the input string into smaller pieces with overlap (to retain context).
fn chunk_with_overlap(s: &str) -> Result<Vec<String>, Error> {
    let (ptr, size) = unsafe { allocator::string_to_ptr(s) };
    let chunks = unsafe { _chunk_with_overlap(ptr, size) };

    let (out_ptr, out_size) = allocator::read_ptr_len(chunks);

    read_chunk_result!(out_ptr, out_size)
}

/// Chunks the input string into smaller pieces by sentence.
fn chunk_by_sentence(s: &str) -> Result<Vec<String>, Error> {
    let (ptr, size) = unsafe { allocator::string_to_ptr(s) };
    let chunks = unsafe { _chunk_by_sentence(ptr, size) };

    let (out_ptr, out_size) = allocator::read_ptr_len(chunks);

    read_chunk_result!(out_ptr, out_size)
}

fn on_create(ptr: u32, len: u32) -> Result<(), Error> {
    // Read the Cap'n Proto message from the provided pointer and length.
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let mut cursor = std::io::Cursor::new(slice);
    let message = capnp::serialize::read_message(&mut cursor, capnp::message::ReaderOptions::new())
        .map_err(Error::Capnp)?;
    let entry = message
        .get_root::<entry_capnp::entry::Reader>()
        .map_err(Error::Capnp)?;

    let name = capnp_str!(entry.get_name());
    let url = capnp_str!(entry.get_url());

    debug(&format!("Entry {{ name: {}, url: {} }}", name, url));
    Ok(())
}

/// The main function for the `on_create` event.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "on_create"))]
pub unsafe extern "C" fn _on_create(ptr: u32, len: u32) {
    if let Err(e) = on_create(ptr, len) {
        error(&format!("Error in on_create: {:?}", e));
    }
}
