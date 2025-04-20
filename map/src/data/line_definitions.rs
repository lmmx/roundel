#[derive(Debug, Clone, Copy)]
pub enum LineType {
    Underground,
    Overground,
    DLR,
    ElizabethLine,
    Tram,
    CableCar,
    Thameslink,
}

#[derive(Debug, Clone)]
pub struct LineInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub color: &'static str,
    pub line_type: LineType,
}

pub const LINE_INFOS: &[LineInfo] = &[
    LineInfo {
        id: "bakerloo",
        name: "Bakerloo",
        color: "#B36305",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "central",
        name: "Central",
        color: "#E32017",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "circle",
        name: "Circle",
        color: "#FFD300",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "district",
        name: "District",
        color: "#00782A",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "hammersmith-city",
        name: "Hammersmith & City",
        color: "#F3A9BB",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "jubilee",
        name: "Jubilee",
        color: "#A0A5A9",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "metropolitan",
        name: "Metropolitan",
        color: "#9B0056",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "northern",
        name: "Northern",
        color: "#000000",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "piccadilly",
        name: "Piccadilly",
        color: "#003688",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "victoria",
        name: "Victoria",
        color: "#0098D4",
        line_type: LineType::Underground,
    },
    LineInfo {
        id: "waterloo-city",
        name: "Waterloo & City",
        color: "#95CDBA",
        line_type: LineType::Underground,
    },
    // LineInfo {
    //     id: "london-overground",
    //     name: "Overground",
    //     color: "#EE7C0E",
    //     line_type: LineType::Overground,
    // },
    LineInfo {
        id: "dlr",
        name: "DLR",
        color: "#00A4A7",
        line_type: LineType::DLR,
    },
    LineInfo {
        id: "elizabeth",
        name: "Elizabeth Line",
        color: "#6950A1",
        line_type: LineType::ElizabethLine,
    },
    LineInfo {
        id: "tram",
        name: "Trams",
        color: "#84B817",
        line_type: LineType::Tram,
    },
    LineInfo {
        id: "cable-car",
        name: "Cable Car",
        color: "#E21836",
        line_type: LineType::CableCar,
    },
    LineInfo {
        id: "thameslink",
        name: "Thameslink",
        color: "#C1007C",
        line_type: LineType::Thameslink,
    },
    // New Overground Lines
    LineInfo {
        id: "liberty",
        name: "Liberty Line",
        color: "#4C6366",
        line_type: LineType::Overground,
    },
    LineInfo {
        id: "lioness",
        name: "Lioness Line",
        color: "#FFA32B",
        line_type: LineType::Overground,
    },
    LineInfo {
        id: "mildmay",
        name: "Mildmay Line",
        color: "#088ECC",
        line_type: LineType::Overground,
    },
    LineInfo {
        id: "suffragette",
        name: "Suffragette Line",
        color: "#59C274",
        line_type: LineType::Overground,
    },
    LineInfo {
        id: "weaver",
        name: "Weaver Line",
        color: "#B43983",
        line_type: LineType::Overground,
    },
    LineInfo {
        id: "windrush",
        name: "Windrush Line",
        color: "#FF2E24",
        line_type: LineType::Overground,
    },
];

// Helper functions
pub fn get_line_color(line_id: &str) -> String {
    LINE_INFOS
        .iter()
        .find(|info| info.id == line_id)
        .map(|info| info.color.to_string())
        .unwrap_or_else(|| "#777777".to_string())
}

pub fn get_underground_lines() -> Vec<&'static LineInfo> {
    LINE_INFOS
        .iter()
        .filter(|info| matches!(info.line_type, LineType::Underground))
        .collect()
}

pub fn get_overground_lines() -> Vec<&'static LineInfo> {
    LINE_INFOS
        .iter()
        .filter(|info| matches!(info.line_type, LineType::Overground))
        .collect()
}

pub fn get_other_rail_lines() -> Vec<&'static LineInfo> {
    LINE_INFOS
        .iter()
        .filter(|info| {
            // "Other rail" means neither Underground nor Overground
            !matches!(info.line_type, LineType::Underground | LineType::Overground)
        })
        .collect()
}

/// Generate CSS for the lines
pub fn generate_line_css() -> String {
    let mut css = String::new();

    // Root variables
    css.push_str(":root {\n");
    for line in LINE_INFOS {
        css.push_str(&format!("  --{}: {};\n", line.id, line.color));
    }
    css.push_str("}\n\n");

    // Line classes
    for line in LINE_INFOS {
        css.push_str(&format!(
            ".color-line.{} {{ background-color: var(--{}); }}\n",
            line.id, line.id
        ));
    }

    css
}
