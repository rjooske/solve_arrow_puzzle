use std::str::FromStr;

use serde::Deserialize;

use crate::gui::Point;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub first_arrow_position: Point,
    pub claim_button_position: Point,
    pub arrow_diameter: i64,
}

impl FromStr for Config {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
