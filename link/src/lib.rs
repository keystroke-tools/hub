use hubble::{entry, error::Error, transform, types};

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
    let name = if entry.name.trim().is_empty() {
        entry.url.clone()
    } else {
        entry.name.clone()
    };

    let markdown = transform::url_to_markdown(entry.url.as_ref())
        .map_err(|e| Error::PluginError(format!("Error converting URL to markdown: {}", e)))?;
    let content = transform::md_to_content(&markdown)?;
    let chunkable_content = format!("{}\n{}", name, content.plain_text);

    let chunks = transform::chunk_with_overlap(&chunkable_content)
        .map_err(|e| Error::PluginError(format!("Error chunking markdown: {}", e)))?;

    let language = whatlang::detect_lang(markdown.as_ref())
        .unwrap_or(whatlang::Lang::Eng)
        .to_string()
        .to_lowercase();

    // Update the entry's content
    let checksum = hubble::generate_checksum(markdown.as_ref());
    entry::update(types::UpdateEntryOpts {
        id: entry.id.clone(),
        name: if entry.name.is_empty() {
            Some(name)
        } else {
            None
        },
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
            language: language.clone(),
        });
    }

    let count = entry::create_chunks(types::CreateChunksOpts {
        chunks: entry_chunks,
    })?;

    hubble::log::debug(&format!(
        "{{ \"count\": {}, \"entry_id\": \"{}\", \"language\": \"{}\" }}",
        count, entry.id, language
    ));

    Ok(())
}
