use crate::data::line_definitions::{
    get_other_rail_lines, get_overground_lines, get_underground_lines,
};
use dioxus::prelude::*;

#[component]
pub fn KeyPanel(visible: bool, on_close: EventHandler<()>) -> Element {
    let underground_lines = get_underground_lines();
    let overground_lines = get_overground_lines();
    let other_rail_lines = get_other_rail_lines();

    rsx! {
        div {
            class: if visible { "oim-key-panel visible" } else { "oim-key-panel" },

            div {
                class: "oim-key-header",
                h2 { "Key" }
                button {
                    class: "oim-key-close",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            div {
                class: "oim-key-body",

                h3 { "Underground Lines" }
                table {
                    // Dynamically generate rows for underground lines
                    for line in &underground_lines {
                        tr {
                            td { "{line.name}" }
                            td {
                                div {
                                    class: format_args!("color-line {}", line.id)
                                }
                            }
                        }
                    }
                }

                h3 { "Overground Lines" }
                table {
                    // Dynamically generate rows for overground lines
                    for line in &overground_lines {
                        tr {
                            td { "{line.name}" }
                            td {
                                div {
                                    class: format_args!("color-line {}", line.id)
                                }
                            }
                        }
                    }
                }

                h3 { "Other Lines" }
                table {
                    // Dynamically generate rows for other rail lines
                    for line in &other_rail_lines {
                        tr {
                            td { "{line.name}" }
                            td {
                                div {
                                    class: format_args!("color-line {}", line.id)
                                }
                            }
                        }
                    }
                }

                h3 { "Features" }
                table {
                    tr {
                        td { "Station" }
                        td {
                            div {
                                class: "map-symbol station"
                            }
                        }
                    }
                    tr {
                        td { "Interchange" }
                        td {
                            div {
                                class: "map-symbol interchange"
                            }
                        }
                    }
                }
            }
        }
    }
}
