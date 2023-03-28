use serde::Deserialize;

use crate::gui::Point;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub first_arrow: Point,
    pub claim_button: Point,
    pub arrow_diameter: i64,
}
