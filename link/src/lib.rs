
use hubble::{error::Error, types};

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
    let parsed_url = url::Url::parse(&entry.url).map_err(|e| Error::PluginError(e.to_string()))?;
    let markdown = hubble::transform::url_to_markdown(parsed_url.as_ref());

    hubble::log::debug(&format!("Markdown: {:?}", markdown));

    Ok(())
}
