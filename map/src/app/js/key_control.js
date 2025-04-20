// Implementation of the KeyControl class implementing MapLibre's IControl interface
class KeyControl {
  constructor() {
    this._map = null;
    this._container = null;
    this._button = null;
  }

  onAdd(map) {
    this._map = map;

    // Create container for the control
    this._container = document.createElement('div');
    this._container.className = 'maplibregl-ctrl maplibregl-ctrl-group';

    // Create the button
    this._button = document.createElement('button');
    this._button.className = 'maplibregl-ctrl-icon oim-key-control';
    this._button.type = 'button';
    this._button.title = 'Show map key';
    this._button.setAttribute('aria-label', 'Show map key');

    // Add button to container
    this._container.appendChild(this._button);

    // Add event listeners - call the Rust-defined function if available
    this._button.addEventListener('click', () => {
      if (window.openTflKeyPanel) {
        window.openTflKeyPanel();
      } else {
        console.log("Key button click failed: no openTflKeyPanel exposed to JS on the window");
      }
    });

    return this._container;
  }

  onRemove() {
    if (this._container && this._container.parentNode) {
      this._container.parentNode.removeChild(this._container);
    }
    this._map = null;
  }

  getDefaultPosition() {
    return 'top-right';
  }
}

// Export the control constructor
window.KeyControl = KeyControl;
