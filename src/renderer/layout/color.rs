/// Helper struct and methods to handle color name, color code and RGB value.
use std::string::String;

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    name: Option<String>,
    code: String,
    rgb: (f64, f64, f64),
}

impl Color {
    pub fn from_name(name: &str) -> Self {
        let code = match name {
            "black" => "#000000".to_string(),
            "silver" => "#c0c0c0".to_string(),
            "gray" => "#808080".to_string(),
            "white" => "#ffffff".to_string(),
            "maroon" => "#800000".to_string(),
            "red" => "#ff0000".to_string(),
            "purple" => "#800080".to_string(),
            "fuchsia" => "#ff00ff".to_string(),
            "green" => "#008000".to_string(),
            "lime" => "#00ff00".to_string(),
            "olive" => "#808000".to_string(),
            "yellow" => "#ffff00".to_string(),
            "navy" => "#000080".to_string(),
            "blue" => "#0000ff".to_string(),
            "teal" => "#008080".to_string(),
            "aqua" => "#00ffff".to_string(),
            "orange" => "#ffa500".to_string(),
            "lightgray" => "#d3d3d3".to_string(),
            _ => {
                println!("warning: color name {:?} is not supported yet", name);
                "#ffffff".to_string()
            }
        };

        let rgb = match name {
            "black" => (0.0, 0.0, 0.0),           // #000000
            "silver" => (0.752, 0.752, 0.752),    // #c0c0c0
            "gray" => (0.501, 0.501, 0.501),      // #808080
            "white" => (1.0, 1.0, 1.0),           // #ffffff
            "maroon" => (0.501, 0.0, 0.0),        // #800000
            "red" => (1.0, 0.0, 0.0),             // #ff0000
            "purple" => (0.501, 0.0, 0.501),      // #800080
            "fuchsia" => (1.0, 0.0, 1.0),         // #ff00ff
            "green" => (0.0, 0.501, 0.0),         // #008000
            "lime" => (0.0, 1.0, 0.0),            // #00ff00
            "olive" => (0.501, 0.501, 0.0),       // #808000
            "yellow" => (1.0, 1.0, 0.0),          // #ffff00
            "navy" => (0.0, 0.0, 0.501),          // #000080
            "blue" => (0.0, 0.0, 1.0),            // #0000ff
            "teal" => (0.0, 0.501, 0.501),        // #008080
            "aqua" => (0.0, 1.0, 1.0),            // #00ffff
            "orange" => (1.0, 0.647, 0.0),        // #ffa500
            "lightgray" => (0.827, 0.827, 0.827), // #d3d3d3
            _ => {
                println!("warning: color name {:?} is not supported yet", name);
                (1.0, 1.0, 1.0)
            }
        };

        Self {
            name: Some(name.to_string()),
            code,
            rgb,
        }
    }

    pub fn from_code(code: &str) -> Self {
        if code.chars().nth(0) != Some('#') || code.len() != 7 {
            // TODO: support color code with 4 chars such as "#fff".
            panic!("invalid color code {}", code);
        }

        let name = match code {
            "#000000" => "black".to_string(),
            "#c0c0c0" => "silver".to_string(),
            "#808080" => "gray".to_string(),
            "#ffffff" => "white".to_string(),
            "#800000" => "maroon".to_string(),
            "#ff0000" => "red".to_string(),
            "#800080" => "purple".to_string(),
            "#ff00ff" => "fuchsia".to_string(),
            "#008000" => "green".to_string(),
            "#00ff00" => "lime".to_string(),
            "#808000" => "olive".to_string(),
            "#ffff00" => "yellow".to_string(),
            "#000080" => "navy".to_string(),
            "#0000ff" => "blue".to_string(),
            "#008080" => "teal".to_string(),
            "#00ffff" => "aqua".to_string(),
            "#ffa500" => "orange".to_string(),
            "#d3d3d3" => "lightgray".to_string(),
            _ => {
                println!("warning: color code {:?} is not supported yet", code);
                "white".to_string()
            }
        };

        let r =
            (u64::from_str_radix(&code[1..3], 16).expect("failed to parse int") as f64) / 255f64;
        let g =
            (u64::from_str_radix(&code[3..5], 16).expect("failed to parse int") as f64) / 255f64;
        let b =
            (u64::from_str_radix(&code[5..7], 16).expect("failed to parse int") as f64) / 255f64;

        Self {
            name: Some(name),
            code: code.to_string(),
            rgb: (r, g, b),
        }
    }

    pub fn _from_rgb() -> Self {
        // TODO: implement
        Self {
            name: Some("white".to_string()),
            code: "#ffffff".to_string(),
            rgb: (0.0, 0.0, 0.0),
        }
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn _code(&self) -> String {
        self.code.clone()
    }

    pub fn rgb(&self) -> (f64, f64, f64) {
        self.rgb
    }
}
