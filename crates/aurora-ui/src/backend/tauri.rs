//! The desktop backend: `window.__TAURI__.core.invoke` and `event.listen` through
//! `wasm-bindgen` (research R1, `app.withGlobalTauri`).

use std::path::PathBuf;

use aurora_core::domain::{EngineReport, EnginesOverview, Settings};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::{CommandError, OnReady, OnReport, Reply};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"], js_name = listen)]
    async fn tauri_listen(
        event: &str,
        handler: &Closure<dyn FnMut(JsValue)>,
    ) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
pub struct AddArgs {
    pub path: PathBuf,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct RenameArgs {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct IdArgs {
    pub id: String,
}

#[derive(Serialize)]
pub struct RelocateArgs {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Serialize)]
pub struct SettingsArgs {
    pub settings: Settings,
}

/// Whether the page runs inside the Tauri window.
pub fn available() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map(|value| !value.is_undefined())
        .unwrap_or(false)
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, CommandError> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(CommandError::internal)
}

fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, CommandError> {
    serde_wasm_bindgen::from_value(value).map_err(CommandError::internal)
}

pub async fn invoke<A: Serialize, T: DeserializeOwned>(command: &str, args: &A) -> Reply<T> {
    let args = to_js(args)?;
    let args = if args.is_null() {
        JsValue::UNDEFINED
    } else {
        args
    };
    match tauri_invoke(command, args).await {
        Ok(value) => from_js(value),
        Err(error) => Err(from_js::<CommandError>(error.clone()).unwrap_or_else(|_| {
            CommandError::internal(error.as_string().unwrap_or_else(|| format!("{error:?}")))
        })),
    }
}

#[derive(Deserialize)]
struct EventEnvelope<T> {
    payload: T,
}

fn listen<T: DeserializeOwned + 'static>(event: &'static str, handle: impl Fn(T) + 'static) {
    let handler = Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
        match serde_wasm_bindgen::from_value::<EventEnvelope<T>>(value) {
            Ok(envelope) => handle(envelope.payload),
            Err(error) => web_sys::console::warn_1(&format!("{event}: {error}").into()),
        }
    });
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = tauri_listen(event, &handler).await {
            web_sys::console::error_1(&error);
        }
        // The listeners live as long as the window.
        handler.forget();
    });
}

pub fn subscribe_engines(on_report: OnReport, on_ready: OnReady) {
    listen::<EngineReport>("engine-report", move |report| on_report(report));
    listen::<EnginesOverview>("engines-ready", move |overview| on_ready(overview));
}
