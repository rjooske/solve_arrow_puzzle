use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn euclidean_distance_to(&self, other: &Color) -> f64 {
        let dr = self.r - other.r;
        let dg = self.g - other.g;
        let db = self.b - other.b;
        let x: f64 = (dr * dr + dg * dg + db * db).into();
        x.sqrt()
    }
}

#[derive(Serialize)]
pub struct ArrowToColor {
    pub up: Color,
    pub right: Color,
    pub down: Color,
    pub left: Color,
}
