mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, link!");
    log("Hello, world!");
}
