use dioxus::prelude::*;

#[component]
pub fn SimulationPanel(
    visible: bool,
    on_close: EventHandler<()>,
    on_toggle: EventHandler<()>,
    on_reset: EventHandler<()>,
    is_paused: bool,
    vehicle_count: Option<usize>,
) -> Element {
    rsx! {
        div {
            class: if visible { "oim-simulation-panel visible" } else { "oim-simulation-panel" },

            div {
                class: "oim-simulation-header",
                h2 { "TfL Vehicle Simulation" }
                button {
                    class: "oim-simulation-close",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            div {
                class: "oim-simulation-body",

                div {
                    class: "simulation-controls",
                    button {
                        id: "play-pause-simulation",
                        onclick: move |_| on_toggle.call(()),
                        if is_paused { "Play" } else { "Pause" }
                    }

                    button {
                        id: "reset-simulation",
                        onclick: move |_| on_reset.call(()),
                        "Reset"
                    }
                }

                // Simulation information
                div {
                    class: "simulation-info",

                    p { "Control the movement of vehicles across the TfL network." }

                    if let Some(count) = vehicle_count {
                        div {
                            class: "vehicle-stats",
                            p { "Active vehicles: {count}" }
                        }
                    }

                    p {
                        class: "simulation-status",
                        "Status: ",
                        span {
                            class: if is_paused { "status-paused" } else { "status-running" },
                            if is_paused { "Paused" } else { "Running" }
                        }
                    }
                }
            }
        }
    }
}
