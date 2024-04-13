use std::{
    array,
    borrow::Cow,
    fmt::Write,
    ops::{Index, IndexMut},
};

use itertools::Itertools;

use self::positions::Position;

const fn indices(positions: [Position; 37]) -> [usize; 37] {
    let mut out = [0; 37];
    let mut i = 0;
    while i < 37 {
        out[i] = positions[i].as_index();
        i += 1;
    }
    out
}

const fn position_to_index(positions: [Position; 37]) -> [[Option<usize>; 7]; 7] {
    let mut out = [[None; 7]; 7];
    let mut i = 0;
    while i < 37 {
        let p = positions[i];
        let (x, y) = p.as_xy();
        out[y][x] = Some(p.as_index());
        i += 1;
    }
    out
}

pub mod positions {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Position((usize, usize));

    impl Position {
        pub const fn as_xy(self) -> (usize, usize) {
            self.0
        }

        pub const fn as_index(self) -> usize {
            let Position((x, y)) = self;
            x + 7 * y
        }
    }

    pub const A0: Position = Position((0, 0));
    pub const A1: Position = Position((1, 0));
    pub const A2: Position = Position((2, 0));
    pub const A3: Position = Position((3, 0));
    pub const B0: Position = Position((0, 1));
    pub const B1: Position = Position((1, 1));
    pub const B2: Position = Position((2, 1));
    pub const B3: Position = Position((3, 1));
    pub const B4: Position = Position((4, 1));
    pub const C0: Position = Position((0, 2));
    pub const C1: Position = Position((1, 2));
    pub const C2: Position = Position((2, 2));
    pub const C3: Position = Position((3, 2));
    pub const C4: Position = Position((4, 2));
    pub const C5: Position = Position((5, 2));
    pub const D0: Position = Position((0, 3));
    pub const D1: Position = Position((1, 3));
    pub const D2: Position = Position((2, 3));
    pub const D3: Position = Position((3, 3));
    pub const D4: Position = Position((4, 3));
    pub const D5: Position = Position((5, 3));
    pub const D6: Position = Position((6, 3));
    pub const E1: Position = Position((1, 4));
    pub const E2: Position = Position((2, 4));
    pub const E3: Position = Position((3, 4));
    pub const E4: Position = Position((4, 4));
    pub const E5: Position = Position((5, 4));
    pub const E6: Position = Position((6, 4));
    pub const F2: Position = Position((2, 5));
    pub const F3: Position = Position((3, 5));
    pub const F4: Position = Position((4, 5));
    pub const F5: Position = Position((5, 5));
    pub const F6: Position = Position((6, 5));
    pub const G3: Position = Position((3, 6));
    pub const G4: Position = Position((4, 6));
    pub const G5: Position = Position((5, 6));
    pub const G6: Position = Position((6, 6));
}

#[derive(Debug, Clone, Eq)]
pub struct Hex<T>([Option<T>; 49]);

impl<T> PartialEq for Hex<T>
where
    T: PartialEq,
{
    /// Only compares the items inside the hexagon.
    fn eq(&self, other: &Self) -> bool {
        self.enumerate()
            .zip(other.enumerate())
            .all(|((a, _), (b, _))| a == b)
    }
}

impl<T> Index<Position> for Hex<T> {
    type Output = T;

    fn index(&self, p: Position) -> &Self::Output {
        unsafe {
            self.0
                .get_unchecked(p.as_index())
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

impl<T> IndexMut<Position> for Hex<T> {
    fn index_mut(&mut self, p: Position) -> &mut Self::Output {
        unsafe {
            self.0
                .get_unchecked_mut(p.as_index())
                .as_mut()
                .unwrap_unchecked()
        }
    }
}

impl<T> Hex<T> {
    /// |          A0
    /// |       B0    A1
    /// |    C0    B1    A2
    /// | D0    C1    B2    A3
    /// |    D1    C2    B3
    /// | E1    D2    C3    B4
    /// |    E2    D3    C4
    /// | F2    E3    D4    C5
    /// |    F3    E4    D5
    /// | G3    F4    E5    D6
    /// |    G4    F5    E6
    /// |       G5    F6
    /// |          G6
    pub const POSITIONS: [Position; 37] = {
        use positions::*;
        [
            A0, A1, A2, A3, B0, B1, B2, B3, B4, C0, C1, C2, C3, C4, C5, D0, D1, D2, D3, D4, D5, D6,
            E1, E2, E3, E4, E5, E6, F2, F3, F4, F5, F6, G3, G4, G5, G6,
        ]
    };
    const INDICES: [usize; 37] = indices(Self::POSITIONS);
    const POSITION_TO_INDEX: [[Option<usize>; 7]; 7] = position_to_index(Self::POSITIONS);

    /// |          D0
    /// |       E1    C0
    /// |    F2    D1    B0
    /// | G3    E2    C1    A0
    /// |    F3    D2    B1
    /// | G4    E3    C2    A1
    /// |    F4    D3    B2
    /// | G5    E4    C3    A2
    /// |    F5    D4    B3
    /// | G6    E5    C4    A3
    /// |    F6    D5    B4
    /// |       E6    C5
    /// |          D6
    const POSITIONS_ROTATED_60: [Position; 37] = {
        use positions::*;
        [
            D0, C0, B0, A0, E1, D1, C1, B1, A1, F2, E2, D2, C2, B2, A2, G3, F3, E3, D3, C3, B3, A3,
            G4, F4, E4, D4, C4, B4, G5, F5, E5, D5, C5, G6, F6, E6, D6,
        ]
    };
    const INDICES_ROTATED_60: [usize; 37] = indices(Self::POSITIONS_ROTATED_60);

    pub fn from_fn<F>(mut f: F) -> Hex<T>
    where
        F: FnMut(usize, usize) -> T,
    {
        let mut hex = Hex(array::from_fn(|_| None));
        for p in Self::POSITIONS {
            let (x, y) = p.as_xy();
            unsafe {
                *hex.0.get_unchecked_mut(p.as_index()) = Some(f(x, y));
            }
        }
        hex
    }

    pub fn try_map_by_ref<F, U, E>(&self, mut f: F) -> Result<Hex<U>, E>
    where
        F: FnMut(&T) -> Result<U, E>,
    {
        let mut hex = Hex(array::from_fn(|_| None));
        for (t, p) in self.enumerate() {
            unsafe {
                *hex.0.get_unchecked_mut(p.as_index()) = Some(f(t)?);
            }
        }
        Ok(hex)
    }

    pub fn at(&self, x: usize, y: usize) -> Option<&T> {
        let i = Self::POSITION_TO_INDEX.get(y)?.get(x).copied()??;
        let t = unsafe { self.0.get_unchecked(i).as_ref().unwrap_unchecked() };
        Some(t)
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let i = Self::POSITION_TO_INDEX.get(y)?.get(x).copied()??;
        let t = unsafe { self.0.get_unchecked_mut(i).as_mut().unwrap_unchecked() };
        Some(t)
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (&T, Position)> + '_ {
        self.0
            .iter()
            .filter_map(|t| t.as_ref())
            .zip(Self::POSITIONS)
    }

    pub fn enumerate_mut(&mut self) -> impl Iterator<Item = (&mut T, Position)> + '_ {
        self.0
            .iter_mut()
            .filter_map(|t| t.as_mut())
            .zip(Self::POSITIONS)
    }

    pub fn rotate_60_cw(&mut self) {
        let mut rotated = Self(array::from_fn(|_| None));
        for (i, j) in Self::INDICES.into_iter().zip(Self::INDICES_ROTATED_60) {
            unsafe {
                *rotated.0.get_unchecked_mut(i) = self.0.get_unchecked_mut(j).take();
            }
        }
        *self = rotated;
    }

    pub fn flip_horizontally(&mut self) {
        let mut flipped = Self(array::from_fn(|_| None));
        for p in Self::POSITIONS {
            let (x, y) = p.as_xy();
            let i = p.as_index();
            let j = y + 7 * x;
            unsafe {
                *flipped.0.get_unchecked_mut(i) = self.0.get_unchecked_mut(j).take();
            }
        }
        *self = flipped;
    }

    pub fn visualize<F>(&self, mut f: F) -> String
    where
        F: FnMut(&T) -> Cow<str>,
    {
        use positions::*;

        // | -- -- -- A0 -- -- --
        // | -- -- B0 -- A1 -- --
        // | -- C0 -- B1 -- A2 --
        // | D0 -- C1 -- B2 -- A3
        // | -- D1 -- C2 -- B3 --
        // | E1 -- D2 -- C3 -- B4
        // | -- E2 -- D3 -- C4 --
        // | F2 -- E3 -- D4 -- C5
        // | -- F3 -- E4 -- D5 --
        // | G3 -- F4 -- E5 -- D6
        // | -- G4 -- F5 -- E6 --
        // | -- -- G5 -- F6 -- --
        // | -- -- -- G6 -- -- --
        const MAYBE_POSITIONS: [Option<Position>; 7 * 13] = [
            None,
            None,
            None,
            Some(A0),
            None,
            None,
            None,
            None,
            None,
            Some(B0),
            None,
            Some(A1),
            None,
            None,
            None,
            Some(C0),
            None,
            Some(B1),
            None,
            Some(A2),
            None,
            Some(D0),
            None,
            Some(C1),
            None,
            Some(B2),
            None,
            Some(A3),
            None,
            Some(D1),
            None,
            Some(C2),
            None,
            Some(B3),
            None,
            Some(E1),
            None,
            Some(D2),
            None,
            Some(C3),
            None,
            Some(B4),
            None,
            Some(E2),
            None,
            Some(D3),
            None,
            Some(C4),
            None,
            Some(F2),
            None,
            Some(E3),
            None,
            Some(D4),
            None,
            Some(C5),
            None,
            Some(F3),
            None,
            Some(E4),
            None,
            Some(D5),
            None,
            Some(G3),
            None,
            Some(F4),
            None,
            Some(E5),
            None,
            Some(D6),
            None,
            Some(G4),
            None,
            Some(F5),
            None,
            Some(E6),
            None,
            None,
            None,
            Some(G5),
            None,
            Some(F6),
            None,
            None,
            None,
            None,
            None,
            Some(G6),
            None,
            None,
            None,
        ];

        let cows = MAYBE_POSITIONS
            .into_iter()
            .map(|p| match p {
                Some(p) => f(&self[p]),
                None => "".into(),
            })
            .collect_vec();
        let longest = cows.iter().map(|c| c.chars().count()).max().unwrap_or(0);

        let mut buf = String::new();
        for (i, cow) in cows.into_iter().enumerate() {
            write!(buf, "{:w$}", cow, w = longest).unwrap();
            if i % 7 != 6 {
                write!(buf, " ").unwrap();
            } else if i / 7 != 12 {
                writeln!(buf).unwrap();
            }
        }
        buf
    }
}
