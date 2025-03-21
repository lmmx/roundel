# Transport Network Simulation (Rust + WebAssembly)

This code simulates a **transport network** with **2,000 vehicles** (buses and trains) in the browser. It uses a **global, thread-local `RefCell`** in Rust to store and update the vehicles’ shared state. Every half-second, each vehicle moves along its route (horizontal lines for buses, vertical lines for trains), and a `<canvas>` is redrawn to reflect their updated positions. The simulation logs the update/draw performance to the browser console.

This simple setup shows how to **store** & **update** thousands of moving vehicles in Rust, **draw** them in an HTML `<canvas>`, and log **performance** in the **browser console**—all while using a single global `RefCell` for shared state. Enjoy tinkering with it!

## Features

1. **Shared State via `RefCell`**
   - We store all vehicle data in a single, global `RefCell<SharedState>`. This allows **safe interior mutability** without passing around mutable references.

2. **2,000 Vehicles**
   - **1,000 buses** across **100 routes** (horizontal lines).
   - **1,000 trains** across **10 routes** (vertical lines).

3. **Canvas Visualization**
   - Each vehicle is drawn as a small circle on an HTML `<canvas>`.
   - Buses move left→right at their assigned horizontal line (`y = route_index * 10`).
   - Trains move top→bottom at their assigned vertical line (`x = route_index * 10`).
   - When a vehicle reaches the end, it wraps around to the start.

4. **Periodic Updates**
   - A Rust closure (scheduled via `setInterval`) updates all vehicles every 500 ms.
   - The code measures and logs how long each update cycle (and canvas redraw) takes.

5. **Performance Logging**
   - Check the **browser console** to see messages like `"Update & draw took X.XXX ms"`.

## How It Works

1. **Vehicle Initialization**
   - On startup, 2,000 vehicles (buses/trains) are created with random initial positions and speeds.

2. **Vehicle Movement**
   - Each vehicle has:
     - A `vehicle_type` (Bus or Train).
     - A `route_index`, which determines which line they move along.
     - A `position` in the range [0..1], which maps to 0..1000 on the canvas.
     - A `speed` that advances `position` each cycle.
   - If `position` exceeds 1.0, it wraps back around.

3. **Rendering**
   - The `<canvas>` is cleared each cycle.
   - We loop over all vehicles, draw a small circle at `(vehicle.x, vehicle.y)`, then fill it.

4. **Timing & Logging**
   - We note the current time (`js_sys::Date::now()`) before and after updating/drawing.
   - The difference is logged to the console in milliseconds.

## Running the Demo

1. **Install [Trunk](https://trunkrs.dev/)** (or another toolchain that can build and serve Wasm).
2. **Build & Serve**:
   ```sh
   trunk serve
   ```
3. **Open** your browser at the URL Trunk provides (e.g., `http://127.0.0.1:8080`).
4. You should see:
   - A 1000×1000 `<canvas>` with numerous **horizontal and vertical lines** of moving dots.
   - The browser console showing messages like **“Update & draw took 5.000 ms”**.

## File Overview

- **`index.html`**
  Contains the `<canvas>` element and a `<script data-trunk>` tag that points to our Rust/Wasm code.

- **`lib.rs`**
  The main Rust code. Key parts:
  - **`Vehicle`** struct with `vehicle_type`, `route_index`, `position`, `speed`, `x`, `y`.
  - **`SharedState`** struct storing a `Vec<Vehicle>`.
  - **`thread_local!`** for a global `RefCell<SharedState>`.
  - **`init_vehicles()`** to create all vehicles, assign routes and speeds.
  - **`update_all()`** to move each vehicle and recalculate `(x, y)`.
  - A **timer callback** (set via `set_interval_with_callback...`) that updates positions, draws them on `<canvas>`, and logs performance.

## Why `RefCell`?

- Even though this is single-threaded Wasm, using a **global `RefCell`** is a convenient, **Rust-safe** way to share and mutate data across multiple scopes (e.g., callbacks, initialization, rendering) without needing to pass around references.

## Customizing

- **Routes**: You can change the number of bus/train routes or how they’re laid out.
- **Speeds**: Adjust the random generation range if you want faster or slower movement.
- **Drawing**: Swap out circle drawing for images, or scale positions differently for a more complex map layout.
- **Update Frequency**: Change the interval (e.g., from 500 ms to 100 ms) if you want smoother movement.
