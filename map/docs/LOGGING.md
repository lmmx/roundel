# TfL Simulation Logging System

This document describes the centralized logging system implemented for the TfL Simulation application.

## Overview

The logging system provides a standardized way to record and filter application events across different modules. It replaces direct calls to `console::log_1` with a more structured approach that includes log levels, categories, and contextual information.

## Features

- **Log Levels**: Debug, Info, Warning, Error
- **Categorization**: Map, Simulation, UI, App, General
- **Contextual Logging**: Group related logs with source context
- **Conditional Logging**: Only evaluate expensive log messages when needed
- **Runtime Configuration**: Enable/disable specific log levels or categories

## API Reference

### Basic Logging Functions

```rust
// Simple logging at different levels
log::debug("Detailed debugging information");
log::info("General information about application progress");
log::warn("Warning about potential issues");
log::error("Error information when something fails");

// With specific categories
log::debug_with_category(LogCategory::Map, "Map-specific debug info");
log::info_with_category(LogCategory::Simulation, "Simulation-specific info");

// Including source location information
log::debug_with_source("Debug with location", file!(), line!());
// Or using the macro helper:
debug_here!("Debug with location");
```

### Conditional Logging

```rust
// Only evaluates the closure if debug level is enabled
log::debug_enabled(|| {
    let msg = format!("Complex debug info for object: {:#?}", complex_object);
    msg
});
```

### Context-Based Logging

```rust
// Log within a named context
with_context("ComponentName::function_name", LogCategory::Map, |logger| {
    logger.info("Starting operation");
    // ... perform operation ...
    logger.debug("Operation details");
    logger.info("Operation completed");
    // Can return a value from the context
    operation_result
})
```

### Macro Helpers

```rust
// Basic logging macros
debug!("Simple debug message");
info!("Simple info message");
warn!("Simple warning message");
error!("Simple error message");

// With string formatting
debug!("Value is: {}", value);
info!("Processing item {} of {}", current, total);

// Category-specific helpers
debug_map!("Map-specific debug information");
info_map!("Map-specific information");
info_sim!("Simulation-specific information");
```

## Configuration

### Setting Log Levels

```rust
// In main.rs or during initialization
use crate::utils::{set_log_level, LogLevel};

// Development configuration
#[cfg(debug_assertions)]
{
    set_log_level(LogLevel::Debug);
}

// Production configuration
#[cfg(not(debug_assertions))]
{
    set_log_level(LogLevel::Info);
}
```

### Enabling/Disabling Categories

```rust
use crate::utils::{set_category_enabled, LogCategory};

// Disable simulation logs
set_category_enabled(LogCategory::Simulation, false);

// Re-enable simulation logs
set_category_enabled(LogCategory::Simulation, true);
```

## Best Practices

1. **Use Appropriate Log Levels**:
   - `Debug`: Detailed information for diagnosing problems
   - `Info`: Confirmation that things are working as expected
   - `Warn`: Indication that something unexpected happened, but the application can continue
   - `Error`: When something has failed

2. **Categorize Logs**:
   - Use the appropriate category for each log message
   - Create new categories if needed for specific modules

3. **Use Context Logging**:
   - When multiple log messages are related, use `with_context` to group them together
   - Include the class/function name in the context to make logs more traceable

4. **Performance Considerations**:
   - Use `debug_enabled` for expensive-to-format debug messages
   - Keep in mind that even disabled logs incur some minimal overhead

5. **Source Locations**:
   - Include source locations (`file!()`, `line!()`) for critical debug information
   - Use `debug_here!` macro for convenience

## Real-World Examples

### Map Initialization

```rust
pub fn create_map(&mut self, container_id: &str) -> Result<(), JsValue> {
    with_context("MapLibreManager::create_map", LogCategory::Map, |logger| {
        logger.info(&format!("Creating map in container '{}'", container_id));

        // First check if maplibregl is loaded
        Self::debug_check_maplibregl()?;

        // Create the map
        logger.debug("Creating new Map instance");
        let map = Map::new(&create_map_options(container_id)?);

        // Store the map
        self.map = Some(map);
        logger.info("Map created successfully");

        Ok(())
    })
}
```

### Error Handling

```rust
if let Err(err) = operation() {
    log::error_with_category(
        LogCategory::Simulation,
        &format!("Failed to perform operation: {:?}", err)
    );
    return Err(err);
}
```

## Log Output Format

Logs are output to the browser console in the following format:

```
[LEVEL] [CATEGORY] [CONTEXT] Message
```

For example:
```
[INFO] [MAP] [MapLibreManager::create_map] Creating map in container 'maplibre-canvas'
[DEBUG] [MAP] [MapLibreManager::create_map] Map options created successfully
[ERROR] [SIM] Failed to initialize simulation: TypeError: undefined is not a function
```

## Extending the Logging System

To add new categories or features:

1. Update the `LogCategory` enum in `src/utils/log.rs`
2. Add appropriate helper macros or functions
3. Update the `ENABLED_CATEGORIES` array size if adding new categories
