use super::TflLayers;
use dioxus::prelude::*;
use web_sys::window;

// Add this helper function in src/app/layer_panel.rs or at the module level in src/app/mod.rs
fn update_js_layer_visibility(layer_id: &str, visible: bool) {
    if let Some(window) = window() {
        let js_code = format!(
            r#"
            if (window.LayerSwitcher && window.LayerSwitcher.getInstance()) {{
                const layerSwitcher = window.LayerSwitcher.getInstance();
                if (layerSwitcher) {{
                    layerSwitcher.setVisibility("{0}", {1});
                }}
            }}
            "#,
            layer_id, visible
        );

        let _ = js_sys::eval(&js_code);
    }
}

#[component]
pub fn LayerPanel(
    visible: bool,
    layers: Signal<TflLayers>,
    load_bus_routes: Signal<bool>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: if visible { "layer-switcher-list active" } else { "layer-switcher-list" },

            h3 { "Layers" }

            h4 { "Transport" }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "tube",
                    name: "tube",
                    checked: layers.read().tube,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.tube = !updated.tube;
                        layers.set(updated);

                        // Update JavaScript layer visibility for all tube lines
                        update_js_layer_visibility("tube-central", updated.tube);
                        update_js_layer_visibility("tube-northern", updated.tube);
                        update_js_layer_visibility("tube-victoria", updated.tube);
                        update_js_layer_visibility("tube-district", updated.tube);
                        update_js_layer_visibility("tube-bakerloo", updated.tube);
                        update_js_layer_visibility("tube-hammersmith-city", updated.tube);
                        update_js_layer_visibility("tube-piccadilly", updated.tube);
                        update_js_layer_visibility("tube-jubilee", updated.tube);
                        update_js_layer_visibility("tube-metropolitan", updated.tube);
                        update_js_layer_visibility("tube-circle", updated.tube);
                        update_js_layer_visibility("tube-waterloo-city", updated.tube);
                    }
                }
                label {
                    r#for: "tube",
                    "Underground"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "overground",
                    name: "overground",
                    checked: layers.read().overground,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.overground = !updated.overground;
                        layers.set(updated);

                        // Update JavaScript layer visibility for all tube lines
                        update_js_layer_visibility("overground-liberty", updated.overground);
                        update_js_layer_visibility("overground-lioness", updated.overground);
                        update_js_layer_visibility("overground-mildmay", updated.overground);
                        update_js_layer_visibility("overground-suffragette", updated.overground);
                        update_js_layer_visibility("overground-weaver", updated.overground);
                        update_js_layer_visibility("overground-windrush", updated.overground);
                    }
                }
                label {
                    r#for: "overground",
                    "Overground"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "dlr",
                    name: "dlr",
                    checked: layers.read().dlr,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.dlr = !updated.dlr;
                        layers.set(updated);

                        update_js_layer_visibility("dlr", updated.dlr);
                    }
                }
                label {
                    r#for: "dlr",
                    "DLR"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "elizabeth_line",
                    name: "elizabeth_line",
                    checked: layers.read().elizabeth_line,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.elizabeth_line = !updated.elizabeth_line;
                        layers.set(updated);

                        update_js_layer_visibility("elizabeth", updated.elizabeth_line);
                    }
                }
                label {
                    r#for: "elizabeth_line",
                    "Elizabeth Line"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "thameslink",
                    name: "thameslink",
                    checked: layers.read().thameslink,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.thameslink = !updated.thameslink;
                        layers.set(updated);

                        update_js_layer_visibility("thameslink", updated.thameslink);
                    }
                }
                label {
                    r#for: "thameslink",
                    "Thameslink"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "buses",
                    name: "buses",
                    checked: layers.read().buses,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.buses = !updated.buses;
                        layers.set(updated);

                        update_js_layer_visibility("bus", updated.buses);
                    }
                }
                label {
                    r#for: "buses",
                    "Buses"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "trams",
                    name: "trams",
                    checked: layers.read().trams,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.trams = !updated.trams;
                        layers.set(updated);

                        update_js_layer_visibility("tram", updated.trams);
                    }
                }
                label {
                    r#for: "trams",
                    "Trams"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "cable_car",
                    name: "cable_car",
                    checked: layers.read().cable_car,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.cable_car = !updated.cable_car;
                        layers.set(updated);

                        update_js_layer_visibility("cable-car", updated.cable_car);
                    }
                }
                label {
                    r#for: "cable_car",
                    "Cable Car"
                }
            }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "load_bus_routes",
                    name: "load_bus_routes",
                    checked: *load_bus_routes.read(),
                    onchange: move |_| {
                        let current_value = *load_bus_routes.read();
                        load_bus_routes.set(!current_value);
                    }
                }
                label {
                    r#for: "load_bus_routes",
                    "Load Bus Routes (requires reload)"
                }
            }

            h4 { "Infrastructure" }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "stations",
                    name: "stations",
                    checked: layers.read().stations,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.stations = !updated.stations;
                        layers.set(updated);

                        update_js_layer_visibility("stations", updated.stations);
                        update_js_layer_visibility("station-labels", updated.stations);
                    }
                }
                label {
                    r#for: "stations",
                    "Stations"
                }
            }

            h4 { "Background" }
            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "nighttime_lights",
                    name: "nighttime_lights"
                }
                label {
                    r#for: "nighttime_lights",
                    "Nighttime Lights"
                }
            }
            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "labels",
                    name: "labels",
                    checked: false,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.labels = !updated.labels;
                        layers.set(updated);

                        // Update JavaScript layer visibility for all tube lines
                        update_js_layer_visibility("labels-other", updated.labels);
                        update_js_layer_visibility("labels-village", updated.labels);
                        update_js_layer_visibility("labels-town", updated.labels);
                        update_js_layer_visibility("labels-state", updated.labels);
                        update_js_layer_visibility("labels-city", updated.labels);
                        update_js_layer_visibility("labels-city_capital", updated.labels);
                        update_js_layer_visibility("labels-country_3", updated.labels);
                        update_js_layer_visibility("labels-country_2", updated.labels);
                        update_js_layer_visibility("labels-country_1", updated.labels);
                        update_js_layer_visibility("highway-name-path", updated.labels);
                        update_js_layer_visibility("highway-name-minor", updated.labels);
                        update_js_layer_visibility("highway-name-major", updated.labels);
                        update_js_layer_visibility("highway-shield-non-us", updated.labels);
                    }
                }
                label {
                    r#for: "labels",
                    "Labels"
                }
            }

            h4 { "Simulation" }

            div {
                class: "layer-item",
                input {
                    r#type: "checkbox",
                    id: "simulation",
                    name: "simulation",
                    checked: layers.read().simulation,
                    onchange: move |_| {
                        let mut updated = *layers.read();
                        updated.simulation = !updated.simulation;
                        layers.set(updated);

                        // Update visibility of simulation layers via JS
                        let js_code = format!(
                            r#"
                            if (window.mapInstance) {{
                                const visibility = {} ? 'visible' : 'none';
                                if (window.mapInstance.getLayer('buses-layer')) {{
                                    window.mapInstance.setLayoutProperty('buses-layer', 'visibility', visibility);
                                }}
                                if (window.mapInstance.getLayer('trains-layer')) {{
                                    window.mapInstance.setLayoutProperty('trains-layer', 'visibility', visibility);
                                }}
                            }}
                            "#,
                            updated.simulation
                        );
                        let _ = js_sys::eval(&js_code);
                    }
                }
                label {
                    r#for: "simulation",
                    "Vehicle Simulation"
                }
            }

            button {
                class: "close-button",
                onclick: move |_| on_close.call(()),
                "Close"
            }
        }
    }
}
