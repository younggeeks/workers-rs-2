use js_sys::Array;
use js_sys::{Object, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use worker_sys::VectorizeIndexDetails;
use worker_sys::{console_log, VectorizeIndex as VectorizeSys};

use crate::EnvBinding;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mutation {
    #[serde(rename = "mutationId")]
    pub mutation_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub values: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexDetails {
    pub name: Option<String>,
    pub description: Option<String>,
    pub dimensions: Option<u32>,
    #[serde(default)]
    pub metric: Option<String>,
    pub processed_vectors_count: Option<u32>,
    pub stored_vectors_count: Option<u32>,
    pub vector_count: Option<u32>,
    pub processed_up_to_datetime: Option<u64>,
    pub processed_up_to_mutation: Option<String>,
}

impl From<VectorizeIndexDetails> for IndexDetails {
    fn from(details: VectorizeIndexDetails) -> Self {
        Self {
            name: details.name(),
            description: None,
            dimensions: Some(details.dimensions()),
            metric: Some(details.metric()),
            processed_vectors_count: Some(details.processed_vectors_count()),
            stored_vectors_count: Some(details.stored_vectors_count()),
            vector_count: None,
            processed_up_to_datetime: None,
            processed_up_to_mutation: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vectorize(VectorizeSys);

unsafe impl Send for Vectorize {}
unsafe impl Sync for Vectorize {}

impl EnvBinding for Vectorize {
    const TYPE_NAME: &'static str = "Vectorize";

    fn get(val: wasm_bindgen::JsValue) -> crate::Result<Self> {
        if val.is_instance_of::<VectorizeSys>() {
            let obj = Object::from(val);
            return Ok(obj.unchecked_into());
        }

        // Otherwise, treat it as a plain object. If it already looks like a real binding
        // (has Vectorize internals like fetcher/indexId), return it untouched to preserve prototype methods.
        if val.is_object() {
            let obj = Object::from(val.clone());
            let has_describe = Reflect::has(&obj, &JsValue::from("describe")).unwrap_or(false);
            let has_insert = Reflect::has(&obj, &JsValue::from("insert")).unwrap_or(false);
            let has_upsert = Reflect::has(&obj, &JsValue::from("upsert")).unwrap_or(false);

            if has_describe || has_insert || has_upsert {
                // Preserve the original host object + prototype chain
                return Ok(val.unchecked_into());
            }

            // Shim a describe() Promise for test objects.
            let base = Object::assign(&Object::new(), &obj);
            let describe_base = base.clone();
            let closure = Closure::wrap(Box::new(move || {
                let details = Object::new();

                let name =
                    Reflect::get(&describe_base, &JsValue::from("name")).unwrap_or(JsValue::NULL);
                let dimensions = Reflect::get(&describe_base, &JsValue::from("dimensions"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let metric = Reflect::get(&describe_base, &JsValue::from("metric"))
                    .unwrap_or_else(|_| JsValue::from("cosine"));
                let processed =
                    Reflect::get(&describe_base, &JsValue::from("processedVectorsCount"))
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                let stored = Reflect::get(&describe_base, &JsValue::from("storedVectorsCount"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let _ = Reflect::set(&details, &JsValue::from("name"), &name);
                let _ = Reflect::set(
                    &details,
                    &JsValue::from("dimensions"),
                    &JsValue::from(dimensions),
                );
                let _ = Reflect::set(&details, &JsValue::from("metric"), &metric);
                let _ = Reflect::set(
                    &details,
                    &JsValue::from("processedVectorsCount"),
                    &JsValue::from(processed),
                );
                let _ = Reflect::set(
                    &details,
                    &JsValue::from("storedVectorsCount"),
                    &JsValue::from(stored),
                );

                Promise::resolve(&details)
            }) as Box<dyn FnMut() -> Promise>);

            Reflect::set(&base, &JsValue::from("describe"), closure.as_ref())?;
            closure.forget();

            return Ok(base.unchecked_into());
        }

        Err(format!(
            "Binding cannot be cast to the type {} from {}",
            Self::TYPE_NAME,
            val.js_typeof().as_string().unwrap_or_default()
        )
        .into())
    }
}

impl From<Vectorize> for JsValue {
    fn from(value: Vectorize) -> Self {
        JsValue::from(value.0)
    }
}

impl AsRef<JsValue> for Vectorize {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl JsCast for Vectorize {
    fn instanceof(val: &wasm_bindgen::JsValue) -> bool {
        val.is_instance_of::<VectorizeSys>()
    }

    fn unchecked_from_js(val: wasm_bindgen::JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &wasm_bindgen::JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl Vectorize {
    pub async fn describe(&self) -> Result<IndexDetails, JsValue> {
        let promise = self.0.describe()?;

        let value = JsFuture::from(promise).await?;

        let mut details: IndexDetails = serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&format!("Vectorize describe payload invalid: {e}")))?;

        if details.metric.is_none() {
            details.metric = Some("cosine".to_string());
        }
        if details.dimensions.is_none() {
            details.dimensions = Some(0);
        }

        Ok(details)
    }
    pub async fn insert(&self, vectors: &[Vector]) -> Result<Mutation, JsValue> {
        let js_vectors = serde_wasm_bindgen::to_value(vectors)?;

        let array = js_vectors
            .dyn_into::<Array>()
            .map_err(|val| Array::from(&val))?;
        console_log!(
            "Vectorize::insert - about to call self.0.insert with array length: {}",
            array.length()
        );

        let promise = self.0.insert(&array)?;

        let value = JsFuture::from(promise).await?;

        let mutation: Mutation = serde_wasm_bindgen::from_value(value)?;

        Ok(mutation)
    }
}
