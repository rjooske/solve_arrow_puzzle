use crate::puzzle::{Arrow, Board, BoardPoke, Row, RowPoke};
use scrap::Capturer;
use serde::{Deserialize, Serialize};
use std::{io, str::FromStr, thread::sleep, time::Duration, usize};

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
    pub fn euclidean_distance_to(self, other: Color) -> f64 {
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
        .map(|(a, c)| (a, c, c.euclidean_distance_to(target)))
        .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b))
        .map(|(a, _, _)| a)
        .expect("no way the iterator is empty")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenBuf {
    frame: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl ScreenBuf {
    pub fn as_view(&self) -> ScreenView {
        ScreenView {
            frame: &self.frame,
            width: self.width,
            height: self.height,
        }
    }
}

pub struct ScreenView<'a> {
    frame: &'a [u8],
    pub width: usize,
    pub height: usize,
}

impl<'a> ScreenView<'a> {
    fn nth_pixel_apple_silicon(&self, n: usize) -> Option<Color> {
        if n >= self.width * self.height {
            None
        } else {
            match self.frame.get((4 * n)..(4 * (n + 1))) {
                Some(&[b, g, r, _]) => Some(Color { r, g, b }),
                x => panic!("bad {}th pixel: {:?}", n, x),
            }
        }
    }

    pub fn at_apple_silicon(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.nth_pixel_apple_silicon(x + self.width * y)
        }
    }

    pub fn to_buf(&self) -> ScreenBuf {
        ScreenBuf {
            frame: self.frame.to_vec(),
            width: self.width,
            height: self.height,
        }
    }
}

pub struct Screen(Capturer);

impl Screen {
    pub fn new(capturer: Capturer) -> Screen {
        Screen(capturer)
    }

    pub fn view_and_map<F, T>(&mut self, f: F) -> io::Result<T>
    where
        F: FnOnce(ScreenView) -> T,
    {
        let Screen(capturer) = self;
        let width = capturer.width();
        let height = capturer.height();

        loop {
            match capturer.frame() {
                Ok(frame) => {
                    return Ok(f(ScreenView {
                        frame: &frame,
                        width,
                        height,
                    }));
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
}

// #[derive(Debug, Error)]
// pub enum DetectBoardError {
//     #[error("tried to find an arrow outside of the screen")]
//     ArrowOutsideScreenshot,
// }
//
// pub fn detect_board(
//     dim: &Dimensions,
//     atc: &ArrowToColor,
//     s: &ScreenView,
// ) -> Result<Board, DetectBoardError> {
//     let pokes = [RowPoke::A, RowPoke::B, RowPoke::C, RowPoke::D];
//     let rows: Result<Vec<_>, _> = pokes
//         .into_iter()
//         .map(|y| {
//             let arrows: Result<Vec<_>, _> = pokes
//                 .into_iter()
//                 .map(|x| {
//                     let Point { x, y } = dim.arrow_position(&BoardPoke(x, y));
//                     let color = s.at(x as _, y as _)?;
//                     let arrow = atc.closest(color);
//                     Ok(arrow)
//                 })
//                 .collect();
//             arrows.map(|a| Row(a.try_into().unwrap()))
//         })
//         .collect();
//     rows.map(|r| Board(r.try_into().unwrap()))
// }
