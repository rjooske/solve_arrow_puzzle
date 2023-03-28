use std::str::FromStr;

use serde::Deserialize;

use crate::{gui::Point, puzzle::BoardPoke};

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

impl Config {
    pub fn arrow_position(&self, &BoardPoke(x, y): &BoardPoke) -> Point {
        let x: u8 = x.into();
        let y: u8 = y.into();
        let x: i64 = x.into();
        let y: i64 = y.into();
        Point {
            x: self.first_arrow_position.x + self.arrow_diameter * x,
            y: self.first_arrow_position.y + self.arrow_diameter * y,
        }
    }
}
