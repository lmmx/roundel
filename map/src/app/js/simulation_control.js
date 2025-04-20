// Implementation of SimulationControl class implementing MapLibre's IControl interface
class SimulationControl {
  constructor() {
    this._map = null;
    this._container = null;
  }

  onAdd(map) {
    this._map = map;

    // Create container for the control
    this._container = document.createElement('div');
    this._container.className = 'maplibregl-ctrl maplibregl-ctrl-group';

    // Create the button
    this._button = document.createElement('button');
    this._button.className = 'maplibregl-ctrl-simulation';
    this._button.type = 'button';
    this._button.title = 'Simulation Controls';
    this._button.setAttribute('aria-label', 'Simulation Controls');
    this._button.innerHTML = '▶';

    // Add button to container
    this._container.appendChild(this._button);

    // Add event listener to show the Rust-managed panel
    this._button.addEventListener('click', () => {
      if (window.openTflSimulationPanel) {
        window.openTflSimulationPanel();
      } else {
        console.log("Simulation button click failed: no openTflSimulationPanel exposed to JS on the window");
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
window.SimulationControl = SimulationControl;
