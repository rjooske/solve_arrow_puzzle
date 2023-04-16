use std::{fmt::Display, ops::RangeInclusive};

use itertools::Itertools;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
enum ArrowFromU8Error {
    #[error("want value within [0, 6), but got {0}")]
    OutOfRange(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Arrow(u8);

impl TryFrom<u8> for Arrow {
    type Error = ArrowFromU8Error;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        if x < 6 {
            Ok(Arrow(x))
        } else {
            Err(ArrowFromU8Error::OutOfRange(x))
        }
    }
}

impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Arrow {
    fn rotate_mut(&mut self) {
        self.0 = (self.0 + 1) % 6;
    }

    fn rotate(mut self) -> Arrow {
        self.rotate_mut();
        self
    }
}

#[derive(Debug, Clone, Eq)]
struct Board([Arrow; 81]);

impl PartialEq for Board {
    /// Only compares the arrows inside the hexagon.
    fn eq(&self, other: &Self) -> bool {
        self.arrows().zip(other.arrows()).all(|(a, b)| a == b)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = (1..=13)
            .map(|v| {
                let arrows = (0..(v.min(7)))
                    .filter_map(|h| {
                        let x = 1 + h;
                        let y = v - h;
                        Board::Y_TO_X_RANGE.get(y).and_then(|r| {
                            if r.contains(&x) {
                                Some(self.0[x + 9 * y])
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                let padding = " ".repeat(3 * (4 - arrows.len()));
                let row =
                    arrows.into_iter().map(|a| a.to_string()).join("     ");
                padding + &row
            })
            .join("\n");
        write!(f, "{}", rows)
    }
}

impl Board {
    const Y_TO_X_RANGE: [RangeInclusive<usize>; 9] = [
        9..=9,
        1..=4,
        1..=5,
        1..=6,
        1..=7,
        2..=7,
        3..=7,
        4..=7,
        9..=9,
    ];

    /// Iterates over the arrows inside the hexagon along with its x and y
    /// coordinate.
    fn arrows(&self) -> impl Iterator<Item = (usize, usize, Arrow)> + '_ {
        self.0.iter().enumerate().filter_map(|(i, a)| {
            let x = i % 9;
            let y = i / 9;
            if Board::Y_TO_X_RANGE[y].contains(&x) {
                Some((x, y, *a))
            } else {
                None
            }
        })
    }

    fn aligned(&self) -> bool {
        self.arrows().map(|(_, _, a)| a).all_equal()
    }

    fn aligned_to(&self, a: Arrow) -> bool {
        *self.at(1, 1) == a && self.aligned()
    }

    fn at_mut(&mut self, x: u8, y: u8) -> &mut Arrow {
        let i: usize = (x + 9 * y).into();
        &mut self.0[i]
    }

    fn at(&self, x: u8, y: u8) -> &Arrow {
        let i: usize = (x + 9 * y).into();
        &self.0[i]
    }

    fn poke_mut(&mut self, x: u8, y: u8) {
        if !(1..=7).contains(&x) || !(1..=7).contains(&y) {
            panic!("tried to poke outside the board: ({x}, {y})");
        }

        self.at_mut(x - 1, y - 1).rotate_mut();
        self.at_mut(x, y - 1).rotate_mut();
        self.at_mut(x - 1, y).rotate_mut();
        self.at_mut(x, y).rotate_mut();
        self.at_mut(x + 1, y).rotate_mut();
        self.at_mut(x, y + 1).rotate_mut();
        self.at_mut(x + 1, y + 1).rotate_mut();
    }

    fn poke(&self, x: u8, y: u8) -> Board {
        let mut b = self.clone();
        b.poke_mut(x, y);
        b
    }
}

#[rustfmt::skip]
macro_rules! board {
    (
                 $a1:tt
              $b1:tt $a2:tt
           $c1:tt $b2:tt $a3:tt
        $d1:tt $c2:tt $b3:tt $a4:tt
           $d2:tt $c3:tt $b4:tt
        $e1:tt $d3:tt $c4:tt $b5:tt
           $e2:tt $d4:tt $c5:tt
        $f1:tt $e3:tt $d5:tt $c6:tt
           $f2:tt $e4:tt $d6:tt
        $g1:tt $f3:tt $e5:tt $d7:tt
           $g2:tt $f4:tt $e6:tt
              $g3:tt $f5:tt
                 $g4:tt
    ) => {
        (|| -> Result<Board, ArrowFromU8Error> {
            let xx = Arrow::try_from(0).unwrap();
            let a1 = Arrow::try_from($a1)?;
            let a2 = Arrow::try_from($a2)?;
            let a3 = Arrow::try_from($a3)?;
            let a4 = Arrow::try_from($a4)?;
            let b1 = Arrow::try_from($b1)?;
            let b2 = Arrow::try_from($b2)?;
            let b3 = Arrow::try_from($b3)?;
            let b4 = Arrow::try_from($b4)?;
            let b5 = Arrow::try_from($b5)?;
            let c1 = Arrow::try_from($c1)?;
            let c2 = Arrow::try_from($c2)?;
            let c3 = Arrow::try_from($c3)?;
            let c4 = Arrow::try_from($c4)?;
            let c5 = Arrow::try_from($c5)?;
            let c6 = Arrow::try_from($c6)?;
            let d1 = Arrow::try_from($d1)?;
            let d2 = Arrow::try_from($d2)?;
            let d3 = Arrow::try_from($d3)?;
            let d4 = Arrow::try_from($d4)?;
            let d5 = Arrow::try_from($d5)?;
            let d6 = Arrow::try_from($d6)?;
            let d7 = Arrow::try_from($d7)?;
            let e1 = Arrow::try_from($e1)?;
            let e2 = Arrow::try_from($e2)?;
            let e3 = Arrow::try_from($e3)?;
            let e4 = Arrow::try_from($e4)?;
            let e5 = Arrow::try_from($e5)?;
            let e6 = Arrow::try_from($e6)?;
            let f1 = Arrow::try_from($f1)?;
            let f2 = Arrow::try_from($f2)?;
            let f3 = Arrow::try_from($f3)?;
            let f4 = Arrow::try_from($f4)?;
            let f5 = Arrow::try_from($f5)?;
            let g1 = Arrow::try_from($g1)?;
            let g2 = Arrow::try_from($g2)?;
            let g3 = Arrow::try_from($g3)?;
            let g4 = Arrow::try_from($g4)?;

            Ok(Board([
                xx, xx, xx, xx, xx, xx, xx, xx, xx,
                xx, a1, a2, a3, a4, xx, xx, xx, xx,
                xx, b1, b2, b3, b4, b5, xx, xx, xx,
                xx, c1, c2, c3, c4, c5, c6, xx, xx,
                xx, d1, d2, d3, d4, d5, d6, d7, xx,
                xx, xx, e1, e2, e3, e4, e5, e6, xx,
                xx, xx, xx, f1, f2, f3, f4, f5, xx,
                xx, xx, xx, xx, g1, g2, g3, g4, xx,
                xx, xx, xx, xx, xx, xx, xx, xx, xx,
            ]))
        })()
    };
}

#[cfg(test)]
mod board_tests {
    use super::*;

    #[test]
    fn only_compare_arrows_inside_hexagon() {
        let a = board!(
                     1
                  1     1
               1     1     1
            1     1     1     1
               1     1     1
            1     1     1     1
               1     1     1
            1     1     1     1
               1     1     1
            1     1     1     1
               1     1     1
                  1     1
                     1
        )
        .unwrap();
        let b = Board([Arrow::try_from(1).unwrap(); 81]);
        assert_eq!(a, b);
    }

    #[test]
    fn not_aligned() {
        let b = board!(
                     0
                  2     2
               0     0     0
            2     2     2     2
               0     0     0
            2     2     2     2
               0     0     0
            2     2     2     2
               0     0     0
            2     2     2     2
               0     0     0
                  2     2
                     0
        )
        .unwrap();
        assert!(!b.aligned())
    }

    #[test]
    fn aligned() {
        let b = board!(
                     3
                  3     3
               3     3     3
            3     3     3     3
               3     3     3
            3     3     3     3
               3     3     3
            3     3     3     3
               3     3     3
            3     3     3     3
               3     3     3
                  3     3
                     3
        )
        .unwrap();
        assert!(b.aligned())
    }

    #[test]
    fn aligned_to_0() {
        let b = board!(
                     0
                  0     0
               0     0     0
            0     0     0     0
               0     0     0
            0     0     0     0
               0     0     0
            0     0     0     0
               0     0     0
            0     0     0     0
               0     0     0
                  0     0
                     0
        )
        .unwrap();
        assert!(b.aligned_to(Arrow::try_from(0).unwrap()))
    }

    #[test]
    fn poke_center() {
        let got = board!(
                     4
                  4     4
               4     4     4
            4     4     4     4
               4     4     4
            4     4     4     4
               4     4     4
            4     4     4     4
               4     4     4
            4     4     4     4
               4     4     4
                  4     4
                     4
        )
        .unwrap()
        .poke(4, 4);
        let want = board!(
                     4
                  4     4
               4     4     4
            4     4     4     4
               4     5     4
            4     5     5     4
               4     5     4
            4     5     5     4
               4     5     4
            4     4     4     4
               4     4     4
                  4     4
                     4
        )
        .unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn poke_edges() {
        let got = board!(
                     0
                  0     0
               0     0     0
            0     0     0     0
               0     0     0
            0     0     0     0
               0     0     0
            0     0     0     0
               0     0     0
            0     0     0     0
               0     0     0
                  0     0
                     0
        )
        .unwrap()
        .poke(1, 1)
        .poke(2, 5)
        .poke(3, 6)
        .poke(5, 2)
        .poke(7, 4)
        .poke(7, 6);
        let want = board!(
                     1
                  1     1
               0     1     0
            1     0     0     1
               1     0     1
            2     0     0     1
               2     0     1
            2     0     0     2
               1     0     1
            1     0     1     1
               0     1     2
                  0     1
                     1
        )
        .unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn poke_wraparound() {
        let got = board!(
                     5
                  5     5
               5     5     5
            5     5     5     5
               5     5     5
            5     5     5     5
               5     5     5
            5     5     5     5
               5     5     5
            5     5     5     5
               5     5     5
                  5     5
                     5
        )
        .unwrap()
        .poke(3, 3)
        .poke(3, 4);
        let want = board!(
                     5
                  5     5
               5     0     5
            5     1     0     5
               0     1     5
            5     1     0     5
               0     1     5
            5     0     5     5
               5     5     5
            5     5     5     5
               5     5     5
                  5     5
                     5
        )
        .unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn visualize_as_hexagon() {
        let got = Board([Arrow::try_from(1).unwrap(); 81]).to_string();
        let want = r#"
         1
      1     1
   1     1     1
1     1     1     1
   1     1     1
1     1     1     1
   1     1     1
1     1     1     1
   1     1     1
1     1     1     1
   1     1     1
      1     1
         1
"#
        .strip_prefix('\n')
        .unwrap()
        .strip_suffix('\n')
        .unwrap()
        .to_owned();
        assert_eq!(got, want);
    }

    #[test]
    fn visualize_arrows_in_correct_positions() {
        let got = board!(
                     0
                  0     1
               0     1     2
            0     1     2     3
               1     2     3
            1     2     3     4
               2     3     4
            2     3     4     5
               3     4     5
            3     4     5     0
               4     5     0
                  5     0
                     0
        )
        .unwrap()
        .to_string();
        let want = r#"
         0
      0     1
   0     1     2
0     1     2     3
   1     2     3
1     2     3     4
   2     3     4
2     3     4     5
   3     4     5
3     4     5     0
   4     5     0
      5     0
         0
"#
        .strip_prefix('\n')
        .unwrap()
        .strip_suffix('\n')
        .unwrap()
        .to_owned();
        assert_eq!(got, want);
    }
}

#[cfg(test)]
mod board_macro_tests {
    use super::*;

    #[test]
    fn fail_if_any_arrows_are_invalid() {
        let got = board!(
                     3
                  3     3
               3     3     3
            3     3     3     3
               3     3     3
            3     3     3     3
               3     3     3
            3     3     3     3
               3     3     9 // <- Invalid
            3     3     3     3
               3     3     3
                  3     3
                     3
        );
        assert_eq!(got, Err(ArrowFromU8Error::OutOfRange(9)));
    }

    #[test]
    fn uniform_board() {
        let a = board!(
                     5
                  5     5
               5     5     5
            5     5     5     5
               5     5     5
            5     5     5     5
               5     5     5
            5     5     5     5
               5     5     5
            5     5     5     5
               5     5     5
                  5     5
                     5
        )
        .unwrap();
        let b = Board([Arrow::try_from(5).unwrap(); 81]);
        assert_eq!(a, b);
    }
}
