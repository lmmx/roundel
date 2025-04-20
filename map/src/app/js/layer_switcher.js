/**
 * Implementation of the LayerSwitcher class for MapLibre GL JS
 * Based on https://github.com/russss/maplibregl-layer-switcher
 */
class LayerSwitcher {
  constructor(layers, title = 'Layers') {
    this._layers = layers;
    this._layerIndex = {};

    // Index all layers for quick lookup
    for (let layer of this.getLayers()) {
      if (this._layerIndex[layer.id]) {
        console.error(`Duplicate layer ID "${layer.id}". Layer IDs must be unique.`);
      }
      this._layerIndex[layer.id] = layer;
    }

    // Initialize visible layers based on their enabled status
    this._visible = this._default_visible = Object.values(this._layerIndex)
      .filter(layer => layer.enabled)
      .map(layer => layer.id);

    // Store instance for retrieval
    LayerSwitcher._instance = this;
  }

  // Add a static method to get the instance
  static getInstance() {
    return LayerSwitcher._instance;
  }

  // Extract flat list of layers from layer groups
  getLayers() {
    const layers = [];
    for (let item of this._layers) {
      if (item instanceof LayerGroup) {
        layers.push(...item.layers);
      } else if (item instanceof Layer) {
        layers.push(item);
      }
    }
    return layers;
  }

  // Set visibility of a specific layer
  setVisibility(layerId, visible) {
    if (visible) {
      if (!this._visible.includes(layerId)) {
        this._visible.push(layerId);
      }
    } else {
      this._visible = this._visible.filter(id => id !== layerId);
    }

    this._updateVisibility();
  }

  // Update visibility of all layers in the map
  _updateVisibility() {
    if (!this._map) {
      return;
    }

    const layers = this._map.getStyle().layers;
    for (let layer of layers) {
      const layerId = layer.id;

      for (let configLayerId in this._layerIndex) {
        const prefix = this._layerIndex[configLayerId].prefix;
        if (layerId.startsWith(prefix)) {
          const visibility = this._visible.includes(configLayerId) ? 'visible' : 'none';
          this._map.setLayoutProperty(layerId, 'visibility', visibility);
        }
      }
    }
  }

  // Set initial visibility in the style before the map is created
  setInitialVisibility(style) {
    for (let layer of style.layers) {
      for (let configLayerId in this._layerIndex) {
        const prefix = this._layerIndex[configLayerId].prefix;
        if (layer.id.startsWith(prefix) && !this._visible.includes(configLayerId)) {
          if (!layer.layout) {
            layer.layout = {};
          }
          layer.layout.visibility = 'none';
        }
      }
    }
  }

  // MapLibre IControl implementation
  onAdd(map) {
    this._map = map;

    // Initialize visibility when the style is loaded
    if (map.isStyleLoaded()) {
      console.log("LAYER SWITCHER ADDING AS MAP STYLE LOADED")
      this._updateVisibility();
    } else {
      map.on('load', () => {
        console.log("LAYER SWITCHER ADDING AS MAP LOADED")
        this._updateVisibility();
      });
    }

    // Create the control button
    const button = document.createElement('button');
    button.className = 'layer-switcher-button';
    button.setAttribute('aria-label', 'Layer Switcher');

    // Set up event listeners - MODIFIED TO CALL RUST FUNCTION
    button.addEventListener('click', () => {
      // Call the Rust function instead of toggling visibility ourselves
      if (window.openTflLayerPanel) {
        window.openTflLayerPanel();
      } else {
        console.log("Layer button click failed: no openTflLayerPanel exposed to JS on the window");
      }
    });

    // Create container for the button
    const controlContainer = document.createElement('div');
    controlContainer.className = 'maplibregl-ctrl maplibregl-ctrl-group layer-switcher';
    controlContainer.appendChild(button);

    return controlContainer;
  }

  onRemove() {
    this._map = undefined;
  }
}

// Layer class for individual layers
class Layer {
  constructor(id, title, prefix, enabled = false) {
    this.id = id;
    this.title = title;
    this.prefix = prefix;
    this.enabled = enabled;
  }
}

// LayerGroup class for groups of layers
class LayerGroup {
  constructor(title, layers) {
    this.title = title;
    this.layers = layers;
  }
}

LayerSwitcher._instance = null;

// Export the control constructor and helper classes
window.LayerSwitcher = LayerSwitcher;
window.Layer = Layer;
window.LayerGroup = LayerGroup;
