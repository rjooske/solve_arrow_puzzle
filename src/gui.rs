use std::{io, str::FromStr, thread::sleep, time::Duration, usize};

use scrap::{Capturer, Frame};
use serde::{Deserialize, Serialize};

use crate::puzzle::Arrow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn euclidean_distance_to(&self, other: &Color) -> f64 {
        let dr = f64::from(self.r) - f64::from(other.r);
        let dg = f64::from(self.g) - f64::from(other.g);
        let db = f64::from(self.b) - f64::from(other.b);
        let x: f64 = dr * dr + dg * dg + db * db;
        x.sqrt()
    }
}

#[derive(Deserialize, Serialize)]
pub struct ArrowToColor {
    pub up: Color,
    pub right: Color,
    pub down: Color,
    pub left: Color,
}

impl FromStr for ArrowToColor {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ArrowToColor {
    pub fn closest(&self, target: &Color) -> Arrow {
        let (arrow, _) = [
            (Arrow::Up, self.up),
            (Arrow::Right, self.right),
            (Arrow::Down, self.down),
            (Arrow::Left, self.left),
        ]
        .into_iter()
        .min_by(|(_, a), (_, b)| {
            let a = a.euclidean_distance_to(target);
            let b = b.euclidean_distance_to(target);
            a.total_cmp(&b)
        })
        .expect("no way the iterator is empty");
        arrow
    }
}

pub struct Screenshot {
    pixels: Vec<Color>,
    pub width: usize,
    pub height: usize,
}

impl Screenshot {
    fn from_frame_apple_silicon(
        f: Frame,
        width: usize,
        height: usize,
    ) -> Screenshot {
        let pixels = f
            .chunks(4)
            .take(width * height)
            .map(|x| match x {
                &[b, g, r, _] => Color { r, g, b },
                _ => panic!("unexpected chunk from frame buffer: {:?}", x),
            })
            .collect();
        Screenshot {
            pixels,
            width,
            height,
        }
    }

    pub fn take(c: &mut Capturer) -> io::Result<Screenshot> {
        let w = c.width();
        let h = c.height();

        loop {
            match c.frame() {
                Ok(f) => {
                    return Ok(Screenshot::from_frame_apple_silicon(f, w, h))
                }
                Err(err) => {
                    if err.kind() == io::ErrorKind::WouldBlock {
                        sleep(Duration::from_millis(1));
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }

    pub fn at(&self, x: usize, y: usize) -> Option<Color> {
        self.pixels.get(x + self.width * y).copied()
    }
}
