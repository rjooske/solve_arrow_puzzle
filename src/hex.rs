use std::{array, borrow::Cow, fmt::Write};

use itertools::Itertools;

const fn index(position: (usize, usize)) -> usize {
    let (x, y) = position;
    x + 7 * y
}

const fn indices(positions: [(usize, usize); 37]) -> [usize; 37] {
    let mut out = [0; 37];
    let mut i = 0;
    while i < 37 {
        let (x, y) = positions[i];
        out[i] = x + 7 * y;
        i += 1;
    }
    out
}

const fn position_to_index(positions: [(usize, usize); 37]) -> [[Option<usize>; 7]; 7] {
    let mut out = [[None; 7]; 7];
    let mut i = 0;
    while i < 37 {
        let (x, y) = positions[i];
        out[y][x] = Some(x + 7 * y);
        i += 1;
    }
    out
}

const A0: (usize, usize) = (0, 0);
const A1: (usize, usize) = (1, 0);
const A2: (usize, usize) = (2, 0);
const A3: (usize, usize) = (3, 0);
const B0: (usize, usize) = (0, 1);
const B1: (usize, usize) = (1, 1);
const B2: (usize, usize) = (2, 1);
const B3: (usize, usize) = (3, 1);
const B4: (usize, usize) = (4, 1);
const C0: (usize, usize) = (0, 2);
const C1: (usize, usize) = (1, 2);
const C2: (usize, usize) = (2, 2);
const C3: (usize, usize) = (3, 2);
const C4: (usize, usize) = (4, 2);
const C5: (usize, usize) = (5, 2);
const D0: (usize, usize) = (0, 3);
const D1: (usize, usize) = (1, 3);
const D2: (usize, usize) = (2, 3);
const D3: (usize, usize) = (3, 3);
const D4: (usize, usize) = (4, 3);
const D5: (usize, usize) = (5, 3);
const D6: (usize, usize) = (6, 3);
const E1: (usize, usize) = (1, 4);
const E2: (usize, usize) = (2, 4);
const E3: (usize, usize) = (3, 4);
const E4: (usize, usize) = (4, 4);
const E5: (usize, usize) = (5, 4);
const E6: (usize, usize) = (6, 4);
const F2: (usize, usize) = (2, 5);
const F3: (usize, usize) = (3, 5);
const F4: (usize, usize) = (4, 5);
const F5: (usize, usize) = (5, 5);
const F6: (usize, usize) = (6, 5);
const G3: (usize, usize) = (3, 6);
const G4: (usize, usize) = (4, 6);
const G5: (usize, usize) = (5, 6);
const G6: (usize, usize) = (6, 6);

const A0_INDEX: usize = index(A0);
const A1_INDEX: usize = index(A1);
const A2_INDEX: usize = index(A2);
const A3_INDEX: usize = index(A3);
const B0_INDEX: usize = index(B0);
const B1_INDEX: usize = index(B1);
const B2_INDEX: usize = index(B2);
const B3_INDEX: usize = index(B3);
const B4_INDEX: usize = index(B4);
const C0_INDEX: usize = index(C0);
const C1_INDEX: usize = index(C1);
const C2_INDEX: usize = index(C2);
const C3_INDEX: usize = index(C3);
const C4_INDEX: usize = index(C4);
const C5_INDEX: usize = index(C5);
const D0_INDEX: usize = index(D0);
const D1_INDEX: usize = index(D1);
const D2_INDEX: usize = index(D2);
const D3_INDEX: usize = index(D3);
const D4_INDEX: usize = index(D4);
const D5_INDEX: usize = index(D5);
const D6_INDEX: usize = index(D6);
const E1_INDEX: usize = index(E1);
const E2_INDEX: usize = index(E2);
const E3_INDEX: usize = index(E3);
const E4_INDEX: usize = index(E4);
const E5_INDEX: usize = index(E5);
const E6_INDEX: usize = index(E6);
const F2_INDEX: usize = index(F2);
const F3_INDEX: usize = index(F3);
const F4_INDEX: usize = index(F4);
const F5_INDEX: usize = index(F5);
const F6_INDEX: usize = index(F6);
const G3_INDEX: usize = index(G3);
const G4_INDEX: usize = index(G4);
const G5_INDEX: usize = index(G5);
const G6_INDEX: usize = index(G6);

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
            .all(|((_, _, a), (_, _, b))| a == b)
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
    pub const POSITIONS: [(usize, usize); 37] = [
        A0, A1, A2, A3, B0, B1, B2, B3, B4, C0, C1, C2, C3, C4, C5, D0, D1, D2, D3, D4, D5, D6, E1,
        E2, E3, E4, E5, E6, F2, F3, F4, F5, F6, G3, G4, G5, G6,
    ];
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
    const POSITIONS_ROTATED_60: [(usize, usize); 37] = [
        D0, C0, B0, A0, E1, D1, C1, B1, A1, F2, E2, D2, C2, B2, A2, G3, F3, E3, D3, C3, B3, A3, G4,
        F4, E4, D4, C4, B4, G5, F5, E5, D5, C5, G6, F6, E6, D6,
    ];
    const INDICES_ROTATED_60: [usize; 37] = indices(Self::POSITIONS_ROTATED_60);

    pub fn from_fn<F>(mut f: F) -> Hex<T>
    where
        F: FnMut(usize, usize) -> T,
    {
        let mut hex = Hex(array::from_fn(|_| None));
        for (i, (x, y)) in Self::INDICES.into_iter().zip(Self::POSITIONS) {
            hex.0[i] = Some(f(x, y));
        }
        hex
    }

    pub fn at(&self, x: usize, y: usize) -> Option<&T> {
        let i = Self::POSITION_TO_INDEX.get(y)?.get(x).copied()??;
        let t = self
            .0
            .get(i)
            .unwrap()
            .as_ref()
            .expect("hexagonal shape not maintained");
        Some(t)
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let i = Self::POSITION_TO_INDEX.get(y)?.get(x).copied()??;
        let t = self
            .0
            .get_mut(i)
            .unwrap()
            .as_mut()
            .expect("hexagonal shape not maintained");
        Some(t)
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (usize, usize, &T)> + '_ {
        self.0
            .iter()
            .filter_map(|t| t.as_ref())
            .zip(Self::POSITIONS)
            .map(|(t, (x, y))| (x, y, t))
    }

    pub fn enumerate_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut T)> + '_ {
        self.0
            .iter_mut()
            .filter_map(|t| t.as_mut())
            .zip(Self::POSITIONS)
            .map(|(t, (x, y))| (x, y, t))
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

    pub fn visualize<F>(&self, mut f: F) -> String
    where
        F: FnMut(&T) -> Cow<str>,
    {
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
        const MAYBE_INDICES: [Option<usize>; 7 * 13] = [
            None,
            None,
            None,
            Some(A0_INDEX),
            None,
            None,
            None,
            None,
            None,
            Some(B0_INDEX),
            None,
            Some(A1_INDEX),
            None,
            None,
            None,
            Some(C0_INDEX),
            None,
            Some(B1_INDEX),
            None,
            Some(A2_INDEX),
            None,
            Some(D0_INDEX),
            None,
            Some(C1_INDEX),
            None,
            Some(B2_INDEX),
            None,
            Some(A3_INDEX),
            None,
            Some(D1_INDEX),
            None,
            Some(C2_INDEX),
            None,
            Some(B3_INDEX),
            None,
            Some(E1_INDEX),
            None,
            Some(D2_INDEX),
            None,
            Some(C3_INDEX),
            None,
            Some(B4_INDEX),
            None,
            Some(E2_INDEX),
            None,
            Some(D3_INDEX),
            None,
            Some(C4_INDEX),
            None,
            Some(F2_INDEX),
            None,
            Some(E3_INDEX),
            None,
            Some(D4_INDEX),
            None,
            Some(C5_INDEX),
            None,
            Some(F3_INDEX),
            None,
            Some(E4_INDEX),
            None,
            Some(D5_INDEX),
            None,
            Some(G3_INDEX),
            None,
            Some(F4_INDEX),
            None,
            Some(E5_INDEX),
            None,
            Some(D6_INDEX),
            None,
            Some(G4_INDEX),
            None,
            Some(F5_INDEX),
            None,
            Some(E6_INDEX),
            None,
            None,
            None,
            Some(G5_INDEX),
            None,
            Some(F6_INDEX),
            None,
            None,
            None,
            None,
            None,
            Some(G6_INDEX),
            None,
            None,
            None,
        ];

        let cows = MAYBE_INDICES
            .into_iter()
            .map(|i| match i {
                Some(i) => f(unsafe { self.0.get_unchecked(i).as_ref().unwrap_unchecked() }),
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
