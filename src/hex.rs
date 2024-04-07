use std::array;

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
    /// Each character represents which row the item belongs to.
    /// |          A
    /// |       B     A
    /// |    C     B     A
    /// | D     C     B     A
    /// |    D     C     B
    /// | E     D     C     B
    /// |    E     D     C
    /// | F     E     D     C
    /// |    F     E     D
    /// | G     F     E     D
    /// |    G     F     E
    /// |       G     F
    /// |          G
    pub const POSITIONS: [(usize, usize); 37] = [
        // row A
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        // row B
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (4, 1),
        // row C
        (0, 2),
        (1, 2),
        (2, 2),
        (3, 2),
        (4, 2),
        (5, 2),
        // row D
        (0, 3),
        (1, 3),
        (2, 3),
        (3, 3),
        (4, 3),
        (5, 3),
        (6, 3),
        // row E
        (1, 4),
        (2, 4),
        (3, 4),
        (4, 4),
        (5, 4),
        (6, 4),
        // row F
        (2, 5),
        (3, 5),
        (4, 5),
        (5, 5),
        (6, 5),
        // row G
        (3, 6),
        (4, 6),
        (5, 6),
        (6, 6),
    ];
    const INDICES: [usize; 37] = indices(Self::POSITIONS);
    const POSITION_TO_INDEX: [[Option<usize>; 7]; 7] = position_to_index(Self::POSITIONS);

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
}
