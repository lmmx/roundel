use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement};

// Define TypeScript-like bindings for MapLibre GL JS

#[wasm_bindgen]
extern "C" {
    // Map class
    #[wasm_bindgen(js_namespace = maplibregl, js_name = Map)]
    pub type Map;

    #[wasm_bindgen(constructor, js_namespace = maplibregl, js_name = Map)]
    pub fn new(options: &JsValue) -> Map;

    #[wasm_bindgen(method, js_name = getContainer)]
    pub fn get_container(this: &Map) -> HtmlElement;

    #[wasm_bindgen(method, js_name = getCanvas)]
    pub fn get_canvas(this: &Map) -> Element;

    #[wasm_bindgen(method)]
    pub fn addControl(this: &Map, control: &JsValue, position: Option<&str>) -> Map;

    #[wasm_bindgen(method, js_name = setLayoutProperty)]
    pub fn set_layout_property(this: &Map, layer_id: &str, name: &str, value: &JsValue) -> Map;

    #[wasm_bindgen(method, js_name = getLayer)]
    pub fn get_layer_raw(this: &Map, id: &str) -> JsValue;

    #[wasm_bindgen(method, js_name = addSource)]
    pub fn add_source(this: &Map, id: &str, source: &JsValue) -> Map;

    #[wasm_bindgen(method, js_name = getSource)]
    pub fn get_source_raw(this: &Map, id: &str) -> JsValue;

    #[wasm_bindgen(method, js_name = addLayer)]
    pub fn add_layer(this: &Map, layer: &JsValue) -> Map;

    #[wasm_bindgen(method)]
    pub fn on(this: &Map, event: &str, handler: &Closure<dyn FnMut()>) -> Map;

    #[wasm_bindgen(method)]
    pub fn off(this: &Map, event: &str, handler: &Closure<dyn FnMut()>) -> Map;

    #[wasm_bindgen(method, js_name = isStyleLoaded)]
    pub fn is_style_loaded(this: &Map) -> bool;

    // MapLibre Controls - these are correctly named
    #[wasm_bindgen(js_namespace = maplibregl, js_name = NavigationControl)]
    pub type NavigationControl;

    #[wasm_bindgen(constructor, js_namespace = maplibregl, js_name = NavigationControl)]
    pub fn new() -> NavigationControl;

    #[wasm_bindgen(js_namespace = maplibregl, js_name = ScaleControl)]
    pub type ScaleControl;

    #[wasm_bindgen(constructor, js_namespace = maplibregl, js_name = ScaleControl)]
    pub fn new(options: &JsValue) -> ScaleControl;

    // Our custom controls
    #[wasm_bindgen(js_namespace = window, js_name = KeyControl)]
    pub type KeyControl;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = KeyControl)]
    pub fn new() -> KeyControl;

    #[wasm_bindgen(js_namespace = window, js_name = LayerSwitcher)]
    pub type LayerSwitcher;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = LayerSwitcher)]
    pub fn new(layers: &JsValue, title: &str) -> LayerSwitcher;

    // Layer/Group classes
    #[wasm_bindgen(js_namespace = window, js_name = Layer)]
    pub type Layer;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = Layer)]
    pub fn new(id: &str, title: &str, prefix: &str, enabled: bool) -> Layer;

    #[wasm_bindgen(js_namespace = window, js_name = LayerGroup)]
    pub type LayerGroup;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = LayerGroup)]
    pub fn new(title: &str, layers: &JsValue) -> LayerGroup;

    #[wasm_bindgen(js_namespace = window, js_name = SimulationControl)]
    pub type SimulationControl;

    #[wasm_bindgen(constructor, js_namespace = window, js_name = SimulationControl)]
    pub fn new() -> SimulationControl;
}

impl Map {
    pub fn get_layer(&self, id: &str) -> Option<JsValue> {
        let raw = self.get_layer_raw(id);
        if raw.is_null() || raw.is_undefined() {
            None
        } else {
            Some(raw)
        }
    }

    pub fn get_source(&self, id: &str) -> Option<JsValue> {
        let raw = self.get_source_raw(id);
        if raw.is_null() || raw.is_undefined() {
            None
        } else {
            Some(raw)
        }
    }
}

// Helper to access the global MapLibre instance
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_namespace = window, js_name = mapInstance)]
    pub static MAP_INSTANCE: Option<Map>;
}
