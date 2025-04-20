// src/app/line_css.rs
use crate::data::line_definitions;
use dioxus::prelude::*;

/// Component that renders the dynamic CSS for line colors
#[component]
pub fn LineCss() -> Element {
    // Generate CSS content from line definitions
    let css_content = line_definitions::generate_line_css();

    rsx! {
        // Use Dioxus's document::Style component to inject CSS
        document::Style { {css_content} }
    }
}
