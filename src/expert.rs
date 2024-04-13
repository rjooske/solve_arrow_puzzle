use std::fmt::Display;

use thiserror::Error;

use crate::hex::{positions::Position, Hex};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArrowFromU8Error {
    #[error("want value within [0, 6), but got {0}")]
    OutOfRange(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Arrow(pub u8);

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
    const UP: Arrow = Arrow(0);

    fn rotate(&mut self) {
        self.0 = (self.0 + 1) % 6;
    }

    fn distance_to(self, other: Arrow) -> usize {
        let a: isize = self.0.into();
        let b: isize = other.0.into();
        (b - a).rem_euclid(6).try_into().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board(Hex<Arrow>);

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.0.visualize(|a| {
            match a {
                Arrow(0) => "0 ",
                Arrow(1) => "1 ",
                Arrow(2) => "2 ",
                Arrow(3) => "3 ",
                Arrow(4) => "4 ",
                Arrow(5) => "5 ",
                _ => "? ",
            }
            .into()
        });
        write!(f, "{}", s)
    }
}

impl Board {
    pub fn new(arrows: Hex<Arrow>) -> Board {
        Board(arrows)
    }

    pub fn is_solved(&self) -> bool {
        self.0.enumerate().all(|(&a, _)| a == Arrow::UP)
    }

    fn poke(&mut self, p: Position) {
        const DS: [(i64, i64); 7] = [(-1, -1), (0, -1), (-1, 0), (0, 0), (1, 0), (0, 1), (1, 1)];

        let (x, y) = p.as_xy();
        let x: i64 = x.try_into().unwrap();
        let y: i64 = y.try_into().unwrap();
        let xys = DS.into_iter().flat_map(|(dx, dy)| {
            let x: usize = (x + dx).try_into().ok()?;
            let y: usize = (y + dy).try_into().ok()?;
            Some((x, y))
        });
        for (x, y) in xys {
            if let Some(a) = self.0.at_mut(x, y) {
                a.rotate();
            }
        }
    }

    fn solve_this_orientation(mut self) -> Hex<usize> {
        fn partially_solve(b: &mut Board, poke_counts: &mut Hex<usize>) {
            use crate::hex::positions::*;

            /// Groups
            /// |          G0
            /// |       G0    G0
            /// |    G0    G1    G0
            /// | G0    G1    G1    G0
            /// |    G1    G2    G1
            /// | G1    G2    G2    G1
            /// |    G2    G3    G2
            /// | G2    G3    G3    G2
            /// |    G3    G4    G3
            /// | G3    G4    G4    G3
            /// |    G4    G5    G4
            /// |       G5    G5
            /// |          G6
            const PARTIAL_SOLVE_MOVES: [(Position, Position); 30] = [
                // Align group 0 by poking group 1
                (A0, B1),
                (A1, B2),
                (A2, B3),
                (A3, B4),
                (B0, C1),
                (C0, D1),
                (D0, E1),
                // Align group 1 by poking group 2
                (B1, C2),
                (B2, C3),
                (B3, C4),
                (B4, C5),
                (C1, D2),
                (D1, E2),
                (E1, F2),
                // Align group 2 by poking group 3
                (C2, D3),
                (C3, D4),
                (C4, D5),
                (C5, D6),
                (D2, E3),
                (E2, F3),
                (F2, G3),
                // Align group 3 by poking group 4
                (D3, E4),
                (D4, E5),
                (D5, E6),
                (E3, F4),
                (F3, G4),
                // Align group 4 by poking group 5
                (E4, F5),
                (E5, F6),
                (F4, G5),
                // Align group 5 by poking group 6
                (F5, G6),
            ];

            for (solvee, poke) in PARTIAL_SOLVE_MOVES {
                let solvee = b.0[solvee];
                let poke_count = solvee.distance_to(Arrow::UP);
                poke_counts[poke] += poke_count;
                for _ in 0..poke_count {
                    b.poke(poke);
                }
            }
        }

        fn fixup(b: &mut Board, poke_counts: &mut Hex<usize>) {
            use crate::hex::positions::*;

            let d6 = b.0[D6];
            let e6 = b.0[E6];
            let f6 = b.0[F6];

            let a_poke_count = Arrow::UP.distance_to(e6) + d6.distance_to(Arrow::UP);
            let b_d_poke_count = e6.distance_to(Arrow::UP);
            let c_poke_count = if (d6.0 + f6.0) % 2 == 0 { 0 } else { 3 };

            let fixup_pokes = [
                (A0, a_poke_count),
                (A1, b_d_poke_count),
                (A2, c_poke_count),
                (A3, b_d_poke_count),
            ];
            for (poke, poke_count) in fixup_pokes {
                poke_counts[poke] += poke_count;
                for _ in 0..poke_count {
                    b.poke(poke);
                }
            }
        }

        let mut poke_counts = Hex::from_fn(|_, _| 0);
        partially_solve(&mut self, &mut poke_counts);
        fixup(&mut self, &mut poke_counts);
        partially_solve(&mut self, &mut poke_counts);
        for (n, _) in poke_counts.enumerate_mut() {
            *n %= 6;
        }
        poke_counts
    }

    pub fn solve(self) -> Hex<usize> {
        let mut boards = Vec::with_capacity(12);
        let mut board = self;
        for flipped in [false, true] {
            for rotation in 0..6 {
                boards.push((flipped, rotation, board.clone()));
                board.0.rotate_60_cw();
            }
            board.0.flip_horizontally();
        }

        boards
            .into_iter()
            .map(|(flipped, rotation, board)| {
                let mut poke_counts = board.solve_this_orientation();
                for _ in 0..(6i64 - rotation).rem_euclid(6) {
                    poke_counts.rotate_60_cw();
                }
                if flipped {
                    poke_counts.flip_horizontally();
                }
                poke_counts
            })
            .min_by_key(|p| p.enumerate().map(|(&n, _)| n).sum::<usize>())
            .unwrap()
    }
}
