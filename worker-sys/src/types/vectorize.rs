use js_sys::{Array, Promise};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "VectorizeIndex")]
    #[derive(Debug, Clone, PartialEq)]
    pub type VectorizeIndex;

    #[wasm_bindgen(structural, method, catch, js_name=describe)]
    pub fn describe(this: &VectorizeIndex) -> Result<Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_name=insert)]
    pub fn insert(this: &VectorizeIndex, vectors: &Array) -> Result<Promise, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[wasm_bindgen(extends=::js_sys::Object, js_name=VectorizeIndexDetails)]
    pub type VectorizeIndexDetails;

    #[wasm_bindgen(structural, method, getter, js_name=name)]
    pub fn name(this: &VectorizeIndexDetails) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_name=dimensions)]
    pub fn dimensions(this: &VectorizeIndexDetails) -> u32;

    #[wasm_bindgen(structural, method, getter, js_name=metric)]
    pub fn metric(this: &VectorizeIndexDetails) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=processedVectorsCount)]
    pub fn processed_vectors_count(this: &VectorizeIndexDetails) -> u32;

    #[wasm_bindgen(structural, method, getter, js_name=storedVectorsCount)]
    pub fn stored_vectors_count(this: &VectorizeIndexDetails) -> u32;
}
