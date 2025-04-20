// Event management system for the map
use crate::maplibre::bindings::Map;
use crate::utils::log::{self, LogCategory, with_context};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::window;

/// Manager for map event listeners
pub struct EventManager {
    registered_events: HashMap<String, Vec<Closure<dyn FnMut()>>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            registered_events: HashMap::new(),
        }
    }

    /// Add an event listener to the map
    pub fn add_listener(
        &mut self,
        map: &Map,
        event: &str,
        callback: impl FnMut() + 'static,
    ) -> Result<(), JsValue> {
        with_context("EventManager::add_listener", LogCategory::Map, |logger| {
            logger.debug(&format!("Adding listener for '{}' event", event));

            // Create a closure from the callback
            let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);

            // Register the listener with the map
            map.on(event, &closure);

            // Store the closure so it isn't dropped
            let event_listeners = self.registered_events.entry(event.to_string()).or_default();
            event_listeners.push(closure);

            logger.debug(&format!(
                "Added '{}' listener (now {} total)",
                event,
                event_listeners.len()
            ));

            Ok(())
        })
    }

    /// Add a load event handler that will be called when the map is loaded
    pub fn add_load_handler<F>(&mut self, map: &Map, callback: F) -> Result<(), JsValue>
    where
        F: Fn(Map) -> Result<(), JsValue> + 'static,
    {
        // Create a static listener ID to help with debugging
        static mut LISTENER_ID: usize = 0;
        let listener_id = unsafe {
            LISTENER_ID += 1;
            LISTENER_ID
        };

        with_context(
            "EventManager::add_load_handler",
            LogCategory::Map,
            |logger| {
                logger.debug(&format!("Creating 'load' event listener #{}", listener_id));

                // Setup an onload handler for the map
                let load_handler = Closure::wrap(Box::new(move || {
                    log::info_with_category(
                        LogCategory::Map,
                        &format!("Map 'load' event fired (listener #{})", listener_id),
                    );

                    // Get map instance from window
                    let window = match window() {
                        Some(w) => w,
                        None => {
                            log::error_with_category(
                                LogCategory::Map,
                                "Window not available in load handler",
                            );
                            return;
                        }
                    };

                    log::debug_with_category(LogCategory::Map, "Getting mapInstance from window");
                    let map_instance =
                        match js_sys::Reflect::get(&window, &JsValue::from_str("mapInstance")) {
                            Ok(m) => {
                                if m.is_undefined() {
                                    log::error_with_category(
                                        LogCategory::Map,
                                        "mapInstance is undefined",
                                    );
                                    return;
                                }
                                m
                            }
                            Err(e) => {
                                log::error_with_category(
                                    LogCategory::Map,
                                    &format!("Failed to get mapInstance: {:?}", e),
                                );
                                return;
                            }
                        };

                    // Call the provided callback with the map instance
                    let map: Map = map_instance.clone().into();
                    let result = callback(map);

                    if let Err(err) = result {
                        log::error_with_category(
                            LogCategory::Map,
                            &format!("Error in load handler callback: {:?}", err),
                        );
                    }
                }) as Box<dyn FnMut()>);

                // Add the load event handler
                map.on("load", &load_handler);

                // Store the handler to prevent it from being dropped
                let event_listeners = self
                    .registered_events
                    .entry("load".to_string())
                    .or_default();
                event_listeners.push(load_handler);

                logger.debug("'load' event handler registered and stored");

                Ok(())
            },
        )
    }

    /// Remove a specific listener for an event
    pub fn remove_listener(&mut self, map: &Map, event: &str, index: usize) -> Result<(), JsValue> {
        with_context(
            "EventManager::remove_listener",
            LogCategory::Map,
            |logger| {
                if let Some(listeners) = self.registered_events.get_mut(event) {
                    if index < listeners.len() {
                        // Remove from map
                        map.off(event, &listeners[index]);
                        // Remove from our storage
                        listeners.remove(index);
                        logger.debug(&format!("Removed listener {} for event '{}'", index, event));
                        Ok(())
                    } else {
                        logger.error(&format!(
                            "Listener index {} out of bounds for event '{}'",
                            index, event
                        ));
                        Err(JsValue::from_str("Listener index out of bounds"))
                    }
                } else {
                    logger.error(&format!("No listeners registered for event '{}'", event));
                    Err(JsValue::from_str("No listeners for this event"))
                }
            },
        )
    }

    /// Clear all listeners
    pub fn clear_listeners(&mut self, map: &Map) {
        with_context(
            "EventManager::clear_listeners",
            LogCategory::Map,
            |logger| {
                for (event, listeners) in &self.registered_events {
                    for listener in listeners {
                        map.off(event, listener);
                    }
                    logger.debug(&format!(
                        "Cleared {} listeners for '{}'",
                        listeners.len(),
                        event
                    ));
                }
                self.registered_events.clear();
                logger.info("All event listeners cleared");
            },
        );
    }
}
