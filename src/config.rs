use serde::Deserialize;

use crate::gui::Point;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub first_arrow_position: Point,
    pub claim_button_position: Point,
    pub arrow_diameter: i64,
}
