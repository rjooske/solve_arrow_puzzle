use std::{fmt::Display, iter, str::FromStr};

use itertools::Itertools;

use crate::enumerate_2d::Enumerate2dTrait;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
    fn cw_distance_to(&self, other: &Arrow) -> u8 {
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
                [a, b, c, d] => writeln!(f, "│ {}  {}  {}  {} │", a, b, c, d)?,
                _ => unreachable!(),
            }
        }
        write!(f, "└────────────┘")
    }
}

impl Board {
    /// Stringifies the board with highlight at `p`.
    pub(crate) fn to_string_with_highlight(&self, p: &Poke) -> String {
        let px = p.x.into();
        let py = p.y.into();

        let mut buf = String::new();
        buf += "┌────────────┐\n";
        buf += &self
            .arrows
            .iter()
            .enumerate_2d(4)
            .map(|(_, (ax, ay), arrow)| {
                if ax == px && ay == py {
                    format!("\x1b[7m {} \x1b[0m", arrow)
                } else {
                    format!(" {} ", arrow)
                }
            })
            .chunks(4)
            .into_iter()
            .map(|it| match &it.collect::<Vec<_>>()[..] {
                [a, b, c, d] => format!("│{}{}{}{}│", a, b, c, d),
                _ => unreachable!(),
            })
            .join("\n");
        buf += "\n└────────────┘";
        buf
    }

    fn are_bottom_n_rows_aligned(&self, n: u8) -> bool {
        let n = n as usize;
        self.arrows.iter().skip(n * 4).all_equal()
    }

    fn flip_h(&self) -> Board {
        Board {
            arrows: self
                .arrows
                .chunks(4)
                .flat_map(|row| {
                    let mut row = row.to_vec();
                    row.reverse();
                    row
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub(crate) fn poke(&self, p: &Poke) -> Board {
        let px: i16 = p.x.into();
        let py: i16 = p.y.into();
        let arrows = self
            .arrows
            .iter()
            .enumerate_2d(4)
            .map(|(_, (ax, ay), &arrow)| {
                let ax: i16 = ax.try_into().unwrap();
                let ay: i16 = ay.try_into().unwrap();
                match (px - ax, py - ay) {
                    (-1..=1, -1..=1) => arrow.rotate_cw(),
                    _ => arrow,
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Board { arrows }
    }

    pub(crate) fn poke_many(&self, ps: &[Poke]) -> Board {
        ps.iter().fold(self.clone(), |b, p| b.poke(p))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Poke {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

impl Poke {
    fn flip_h(&self) -> Poke {
        Poke {
            x: 4 - self.x,
            y: self.y,
        }
    }
}

pub(crate) struct Solution(pub(crate) Vec<Poke>);

#[derive(Debug)]
pub(crate) struct Solver<Run>(Run)
where
    Run: FnOnce(&Board) -> Solution;

impl<RunA> Solver<RunA>
where
    RunA: FnOnce(&Board) -> Solution,
{
    pub(crate) fn and_then<F, RunB>(self, f: F) -> Solver<impl FnOnce(&Board) -> Solution>
    where
        F: FnOnce() -> Solver<RunB>,
        RunB: FnOnce(&Board) -> Solution,
    {
        Solver(|board| {
            let Solution(mut pokes1) = self.run(board);
            let Solution(mut pokes2) = f().run(&board.poke_many(&pokes1));
            pokes1.append(&mut pokes2);
            Solution(pokes1)
        })
    }

    pub(crate) fn run(self, b: &Board) -> Solution {
        self.0(b)
    }
}

/// Solves the `n`th row.
/// Bottom `n - 1` rows must already be solved.
/// `n` must be 1, 2, or 3.
pub(crate) fn solve_nth_row(n: u8) -> Solver<impl FnOnce(&Board) -> Solution> {
    Solver(move |b| {
        let mut boards = vec![b.clone()];
        let mut poke_count = 0;

        loop {
            if let Some((i, _)) = boards
                .iter()
                .enumerate()
                .find(|(_, b)| b.are_bottom_n_rows_aligned(n))
            {
                let pokes = (0..poke_count)
                    .map(|nth_poke| {
                        let x = i / 4usize.pow(nth_poke);
                        let x = (x % 4).try_into().unwrap();
                        Poke { x, y: n - 1 }
                    })
                    .collect();
                return Solution(pokes);
            }

            boards = boards
                .iter()
                .flat_map(|b| (0..4).map(|x| b.poke(&Poke { x, y: n - 1 })))
                .collect();
            poke_count += 1;
        }
    })
}

// pub(crate) fn solve_bottom_3_rows(b: Board) -> Solver {
//     solve_nth_row(b, 3)
//         .and_then(|b| solve_nth_row(b, 2))
//         .and_then(|b| solve_nth_row(b, 1))
// }
//
// /// Solves the first row of `b`.
// /// The bottom 3 rows must already be solved.
// fn solve_first_row(board: Board) -> Solver {
//     // ┌────────────┐
//     // │ a  b  c  d │
//     // │ e  _  _  h │
//     // │ _  _  _  _ │
//     // │ _  _  _  _ │
//     // └────────────┘
//
//     fn xxxx(board: Board) -> Solver {
//         let a = board.arrows[0];
//         let e = board.arrows[4];
//         let n = e.cw_distance_to(&a).into();
//         Solver {
//             moves: [Poke { x: 0, y: 2 }, Poke { x: 3, y: 2 }].repeat(n),
//             result: Board { arrows: [a; 16] },
//         }
//     }
//
//     fn xxx_(board: Board) -> Solver {
//         let b = board.arrows[1];
//         let d = board.arrows[3];
//         let e = board.arrows[4];
//         let h = board.arrows[7];
//         let h_to_d = h.cw_distance_to(&d).into();
//         let d_to_b = d.cw_distance_to(&b).into();
//         let b_to_d = b.cw_distance_to(&d).into();
//         let m1 = iter::repeat(Poke { x: 0, y: 2 }).take(h_to_d);
//         let m2 = iter::repeat(Poke { x: 3, y: 2 }).take(h_to_d);
//         let m3 = iter::repeat(Poke { x: 1, y: 2 }).take(d_to_b);
//         let m4 = iter::repeat(Poke { x: 1, y: 0 }).take(b_to_d);
//         let m5 = iter::repeat(Poke { x: 1, y: 3 }).take(b_to_d);
//         // Move {
//         //     moves: m1.chain(m2).chain(m3).chain(m4).chain(m5).collect(),
//         // }
//         todo!()
//     }
//
//     let (a, b, c, d) = match &board.arrows[0..4] {
//         [a, b, c, d] => (a, b, c, d),
//         _ => unreachable!(),
//     };
//
//     if a == b && b == c && c == d {
//         xxxx(board)
//     // } else if a {
//     } else {
//         todo!()
//     }
// }
