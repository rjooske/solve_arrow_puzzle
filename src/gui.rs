use std::{io, str::FromStr, thread::sleep, time::Duration, usize};

use scrap::{Capturer, Frame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::puzzle::{Arrow, Board, BoardPoke, Row, RowPoke};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Point {
    pub x: u64,
    pub y: u64,
}

#[derive(Debug, Deserialize)]
pub struct Dimensions {
    pub first_arrow_position: Point,
    pub claim_button_position: Point,
    pub arrow_diameter: u64,
}

impl FromStr for Dimensions {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl Dimensions {
    pub fn arrow_position(&self, &BoardPoke(x, y): &BoardPoke) -> Point {
        let x: u8 = x.into();
        let y: u8 = y.into();
        let x: u64 = x.into();
        let y: u64 = y.into();
        Point {
            x: self.first_arrow_position.x + self.arrow_diameter * x,
            y: self.first_arrow_position.y + self.arrow_diameter * y,
        }
    }
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

#[derive(Debug, Deserialize, Serialize)]
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
    pub fn closest(&self, target: Color) -> Arrow {
        [
            (Arrow::Up, self.up),
            (Arrow::Right, self.right),
            (Arrow::Down, self.down),
            (Arrow::Left, self.left),
        ]
        .into_iter()
        .map(|(a, c)| (a, c, c.euclidean_distance_to(&target)))
        .min_by(|(_, _, a), (_, _, b)| a.total_cmp(&b))
        .map(|(a, _, _)| a)
        .expect("no way the iterator is empty")
    }
}

#[derive(Debug, Error)]
pub enum ScreenshotAtError {
    #[error("({x}, {y}) is outside of the screenshot ({width}x{height})")]
    OutOfRange {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
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

    pub fn at(&self, x: usize, y: usize) -> Result<Color, ScreenshotAtError> {
        self.pixels.get(x + self.width * y).copied().ok_or({
            ScreenshotAtError::OutOfRange {
                x,
                y,
                width: self.width,
                height: self.height,
            }
        })
    }
}

#[derive(Debug, Error)]
pub enum DetectBoardError {
    #[error("tried to find an arrow outside of the screen: {0}")]
    ArrowOutsideScreenshot(#[from] ScreenshotAtError),
}

pub fn detect_board(
    dim: &Dimensions,
    atc: &ArrowToColor,
    s: &Screenshot,
) -> Result<Board, DetectBoardError> {
    let pokes = [RowPoke::A, RowPoke::B, RowPoke::C, RowPoke::D];
    let rows: Result<Vec<_>, _> = pokes
        .into_iter()
        .map(|y| {
            let arrows: Result<Vec<_>, _> = pokes
                .into_iter()
                .map(|x| {
                    let Point { x, y } = dim.arrow_position(&BoardPoke(x, y));
                    let color = s.at(x as _, y as _)?;
                    let arrow = atc.closest(color);
                    Ok(arrow)
                })
                .collect();
            arrows.map(|a| Row(a.try_into().unwrap()))
        })
        .collect();
    rows.map(|r| Board(r.try_into().unwrap()))
}
