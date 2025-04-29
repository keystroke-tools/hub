use hubble::{
    entry,
    error::Error,
    language, transform,
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
    // Remove the file extension from the name (account for multiple dots)
    let parts: Vec<&str> = name.split('.').collect();
    let name = if parts.len() > 1 {
        parts[..parts.len() - 1].join(".")
    } else {
        name.clone()
    };

    // The plugin has been handed a Minio URL to fetch the file from
    let response = hubble::network::request(RequestOpts {
        method: NetworkMethod::Get,
        url: entry.url.clone(),
        headers: None,
        body: None,
    })?;

    if response.status_code >= 400 {
        return Err(Error::PluginError(format!(
            "Failed to fetch data from {}: {}",
            entry.url, response.status_code
        )));
    }

    // Convert the Vec<u8> to a String
    let body = String::from_utf8(response.body).map_err(|e| {
        Error::PluginError(format!("Failed to convert response body to String: {}", e))
    })?;

    let content = match entry.r#type {
        types::Type::HTML => {
            let md = transform::html_to_markdown(&body)?;
            transform::md_to_content(&md).map_err(|e| {
                Error::PluginError(format!("Failed to convert markdown to content: {}", e))
            })?
        }

        types::Type::Markdown => transform::md_to_content(&body).map_err(|e| {
            Error::PluginError(format!("Failed to convert markdown to content: {}", e))
        })?,

        types::Type::PlainText => types::Content {
            markdown: body.clone(),
            plain_text: body,
        },

        _ => {
            return Err(Error::PluginError(format!(
                "Unsupported entry type: {}",
                entry.r#type
            )));
        }
    };

    let chunks = transform::chunk_with_overlap(&format!("{}\n{}", name, content.plain_text))
        .map_err(|_| Error::PluginError("Failed to convert markdown to chunks".into()))?;

    let checksum = hubble::generate_checksum(content.markdown.as_ref());
    let language = language::detect_lang(&content.plain_text);

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
            language: language.to_string(),
        });
    }

    let count = entry::create_chunks(types::CreateChunksOpts {
        chunks: entry_chunks,
    })?;

    hubble::log::debug(&format!(
        "{{ \"type\": \"{}\", \"count\": {}, \"entry_id\": \"{}\", \"language\": \"{}\" }}",
        count, entry.id, language, entry.r#type
    ));

    Ok(())
}
