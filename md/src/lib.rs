use hubble::{
    entry,
    error::Error,
    transform,
    types::{self, NetworkMethod, RequestOpts},
};

/// The main function for the `on_create` event.
///
/// ## Safety
/// This function is marked as `unsafe` because it interacts with "raw" memory pointers.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "on_create"))]
pub unsafe extern "C" fn _on_create(ptr: u32, len: u32) -> u64 {
    match hubble::types::Entry::read_from_memory(ptr, len) {
        Ok(entry) => match on_create(entry) {
            Ok(_) => 0,
            Err(e) => e.write_to_host(),
        },
        Err(e) => e.write_to_host(),
    }
}

fn on_create(entry: types::Entry) -> Result<(), Error> {
    let name = entry.name;
    let name = name.split('.').next().unwrap_or(&name).to_string();

    hubble::log::debug(&format!("Downloading markdown from {}", entry.url));
    let response = hubble::network::request(RequestOpts {
        method: NetworkMethod::Get,
        url: entry.url.clone(),
        headers: None,
        body: None,
    })?;

    if response.status_code != 200 {
        return Err(Error::PluginError(format!(
            "Failed to fetch data from {}: {}",
            entry.url, response.status_code
        )));
    }

    let raw_data = response.body;
    // Convert the Vec<u8> to a String
    let markdown = String::from_utf8(raw_data)
        .map_err(|_| Error::PluginError("Failed to convert response body to String".into()))?;

    let chunks = transform::chunk_with_overlap(&markdown)
        .map_err(|_| Error::PluginError("Failed to convert markdown to chunks".into()))?;
    let checksum = hubble::generate_checksum(markdown.as_ref());
    let content = transform::md_to_content(&markdown)
        .map_err(|e| Error::PluginError(format!("Failed to convert markdown to content: {}", e)))?;

    entry::update(types::UpdateEntryOpts {
        id: entry.id.to_owned(),
        name: Some(name),
        content: Some(content),
        checksum: Some(checksum),
    })?;

    let mut entry_chunks = vec![];

    for idx in 0..chunks.len() {
        let chunk = chunks
            .get(idx)
            .ok_or_else(|| Error::PluginError("Chunk not found".to_string()))?
            .to_string();

        entry_chunks.push(types::NewChunk {
            entry_id: entry.id.clone(),
            index: idx as i32,
            minimum_version: 1,
            content: chunk,
            language: "english".to_string(), // we will just use english for now
        });
    }

    let count = entry::create_chunks(types::CreateChunksOpts {
        chunks: entry_chunks,
    })?;

    hubble::log::debug(&format!(
        "{{ \"count\": {}, \"entry_id\": \"{}\", \"language\": \"english\" }}",
        count, entry.id
    ));

    Ok(())
}
