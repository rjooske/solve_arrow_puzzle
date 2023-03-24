use std::{fmt::Display, ops::Rem, str::FromStr};

use itertools::Itertools;

use crate::enumerate_2d::Enumerate2dTrait;

#[derive(PartialEq, Eq, Clone, Debug)]
enum Arrow {
    Up,
    Right,
    Down,
    Left,
}

impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Up => '↑',
            Self::Right => '→',
            Self::Down => '↓',
            Self::Left => '←',
        };
        write!(f, "{}", c)
    }
}

impl Arrow {
    fn from_char(c: char) -> Option<Arrow> {
        match c.to_ascii_lowercase() {
            'u' => Some(Arrow::Up),
            'r' => Some(Arrow::Right),
            'd' => Some(Arrow::Down),
            'l' => Some(Arrow::Left),
            _ => None,
        }
    }

    fn rotate_cw(&self) -> Arrow {
        match self {
            Arrow::Up => Arrow::Right,
            Arrow::Right => Arrow::Down,
            Arrow::Down => Arrow::Left,
            Arrow::Left => Arrow::Up,
        }
    }

    /// How many CW rotations are needed to go from `self` to `other`.
    fn distance_cw(&self, other: &Arrow) -> u8 {
        match self {
            Arrow::Up => match other {
                Arrow::Up => 0,
                Arrow::Right => 1,
                Arrow::Down => 2,
                Arrow::Left => 3,
            },
            Arrow::Right => match other {
                Arrow::Up => 3,
                Arrow::Right => 0,
                Arrow::Down => 1,
                Arrow::Left => 2,
            },
            Arrow::Down => match other {
                Arrow::Up => 2,
                Arrow::Right => 3,
                Arrow::Down => 0,
                Arrow::Left => 1,
            },
            Arrow::Left => match other {
                Arrow::Up => 1,
                Arrow::Right => 2,
                Arrow::Down => 3,
                Arrow::Left => 0,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Board {
    arrows: [Arrow; 16],
}

impl FromStr for Board {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arrows: Option<[Arrow; 16]> = s
            .chars()
            .take(16)
            .map(Arrow::from_char)
            .collect::<Option<Vec<_>>>()
            .and_then(|v| v.try_into().ok());
        match arrows {
            Some(arrows) => Ok(Self { arrows }),
            None => Err(()),
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌────────────┐")?;
        for row in self.arrows.chunks(4) {
            match &row {
                &[a, b, c, d] => writeln!(f, "│ {}  {}  {}  {} │", a, b, c, d)?,
                _ => unreachable!(),
            }
        }
        write!(f, "└────────────┘")
    }
}

impl Board {
    fn are_bottom_n_rows_aligned(&self, n: u8) -> bool {
        let n = n as usize;
        self.arrows.iter().skip(n * 4).all_equal()
    }

    pub(crate) fn poke(&self, x: u8, y: u8) -> Self {
        let arrows = self
            .arrows
            .iter()
            .enumerate_2d(4)
            .map(|((ax, ay), arrow)| {
                let dx = x as i64 - ax as i64;
                let dy = y as i64 - ay as i64;
                match (dx, dy) {
                    (-1..=1, -1..=1) => arrow.rotate_cw(),
                    _ => arrow.clone(),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("want exactly 16 elements");
        Self { arrows }
    }
}

#[derive(Debug)]
pub(crate) struct Move {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

#[derive(Debug)]
pub(crate) struct Solution {
    pub(crate) moves: Vec<Move>,
    pub(crate) result: Board,
}

impl Solution {
    fn and_then<F>(self, f: F) -> Self
    where
        F: FnOnce(Board) -> Self,
    {
        let mut s = f(self.result);
        let mut moves = self.moves;
        moves.append(&mut s.moves);
        Solution {
            moves,
            result: s.result,
        }
    }
}

/// Solves the `n`th row of `b`.
/// The bottom `n - 1` rows must already be solved.
/// `n` must be 1, 2, or 3.
fn solve_nth_row(b: Board, n: u8) -> Solution {
    let mut boards = vec![b];
    let mut history = vec![];

    loop {
        if let Some((i, result)) = boards
            .iter()
            .enumerate()
            .find(|(_, b)| b.are_bottom_n_rows_aligned(n))
        {
            let moves = (0..history.len())
                .map(|j| {
                    let x = i / 4usize.pow(j as u32);
                    let x = (x % 4) as u8;
                    Move { x, y: n - 1 }
                })
                .collect();
            return Solution {
                moves,
                result: result.clone(),
            };
        }

        let new_boards: Vec<_> = boards
            .iter()
            .flat_map(|b| (0..4).map(|x| b.poke(x, n - 1)))
            .collect();
        history.push(boards);
        boards = new_boards;
    }
}

pub(crate) fn solve_bottom_3_rows(b: Board) -> Solution {
    solve_nth_row(b, 3)
        .and_then(|b| solve_nth_row(b, 2))
        .and_then(|b| solve_nth_row(b, 1))
}

/// Solves the first row of `b`.
/// The bottom 3 rows must already be solved.
fn solve_first_row(board: Board) -> Solution {
    let (a, b, c, d) = match &board.arrows[0..4] {
        [a, b, c, d] => (a, b, c, d),
        _ => unreachable!(),
    };

    if a == b && b == c && c == d {}

    todo!()
}
