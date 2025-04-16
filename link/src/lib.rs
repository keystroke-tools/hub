use dom_smoothie::{Article, Config, Readability, TextMode};
use hubble::{error::Error, types::entry};

/// The main function for the `on_create` event.
///
/// ## Safety
/// This function is marked as `unsafe` because it interacts with "raw" memory pointers.
#[cfg_attr(all(target_arch = "wasm32"), unsafe(export_name = "on_create"))]
pub unsafe extern "C" fn _on_create(ptr: u32, len: u32) {
    match hubble::types::Entry::read_from_capnp(ptr, len) {
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

fn on_create(entry: entry::Entry) -> Result<(), Error> {
    let parsed_url = url::Url::parse(&entry.url).map_err(|e| Error::PluginError(e.to_string()))?;
    let base_url = parsed_url
        .host_str()
        .ok_or_else(|| Error::PluginError("Invalid domain".to_string()))?;

    // let mut readability = Readability::new(html, document_url, cfg)

    // Read the Cap'n Proto message from the provided pointer and length.
    hubble::log::debug(&format!(
        "Entry {{ link: {}, base_url: {} }}",
        entry.url, base_url
    ));
    Ok(())
}
