use hubble::{entry, error::Error, transform, types};

/// The main function for the `on_create` event.
///
/// ## Safety
/// This function is marked as `unsafe` because it interacts with "raw" memory pointers.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "on_create"))]
pub unsafe extern "C" fn _on_create(ptr: u32, len: u32) {
    match hubble::types::Entry::read_from_memory(ptr, len) {
        Ok(entry) => {
            if let Err(e) = on_create(entry) {
                hubble::log::error(&format!("Error processing entry: {:?}", e));
            }
        }
        Err(e) => {
            hubble::log::error(&format!("Error reading entry: {:?}", e));
        }
    }
}

fn on_create(entry: types::Entry) -> Result<(), Error> {
    let markdown = transform::url_to_markdown(entry.url.as_ref())
        .map_err(|e| Error::PluginError(format!("Error converting URL to markdown: {}", e)))?;
    let chunks = transform::chunk_with_overlap(markdown.as_ref())
        .map_err(|e| Error::PluginError(format!("Error chunking markdown: {}", e)))?;

    let language = whatlang::detect_lang(markdown.as_ref())
        .unwrap_or(whatlang::Lang::Eng)
        .to_string()
        .to_lowercase();

    // Update the entry's content
    let checksum = hubble::generate_checksum(markdown.as_ref());
    entry::update(types::UpdateEntryOpts {
        id: entry.id.clone(),
        name: None,
        content: Some(markdown),
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
            language: language.clone(),
        });
    }

    entry::create_chunks(types::CreateChunksOpts {
        chunks: entry_chunks,
    })?;

    hubble::log::debug(&format!(
        "URL: {:?}, Chunks: {}, Language: {}",
        entry.url,
        chunks.len(),
        language
    ));

    Ok(())
}
