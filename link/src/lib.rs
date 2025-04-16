use std::collections::HashMap;

use dom_smoothie::{Article, Config, Readability, TextMode};
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
    let host = parsed_url
        .host_str()
        .ok_or_else(|| Error::PluginError("Invalid domain".to_string()))?;
    let scheme = parsed_url.scheme();
    let base_url = format!("{}://{}", scheme, host);

    let _cfg = Config {
        max_elements_to_parse: 10_000,
        text_mode: TextMode::Markdown,
        ..Default::default()
    };

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15".to_string());
    let resp = hubble::request(types::RequestOpts {
        method: types::NetworkMethod::Get,
        url: entry.url.clone(),
        headers: Some(headers),
        body: None,
    })?;

    // Convert vector to string
    let body = String::from_utf8(resp.body).map_err(|e| Error::PluginError(e.to_string()))?;

    hubble::log::debug(&format!(
        "Response: {{ base_url: {}, code: {}, body: {} }}",
        base_url, resp.status_code, body
    ));

    // let mut readability = Readability::new(html, Some(&base_url), Some(cfg))
    //     .map_err(|e| Error::PluginError(e.to_string()))?;
    //
    // let article: Article = readability
    //     .parse()
    //     .map_err(|e| Error::PluginError(format!("Failed to parse article: {}", e)))?;

    // Read the Cap'n Proto message from the provided pointer and length.
    // hubble::log::debug(&format!(
    //     "Entry {{ title: {}, byline: {} }}",
    //     article.title,
    //     article.byline.unwrap_or_default()
    // ));
    Ok(())
}
