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

#[wasm_bindgen]
pub fn on_create(_ptr: u32, _len: u32) {
    println!("called `on_create`");
}
