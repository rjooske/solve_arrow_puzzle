use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrow {
    Up,
    Right,
    Down,
    Left,
}

impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl Arrow {
    fn to_char(self) -> char {
        match self {
            Arrow::Up => '↑',
            Arrow::Right => '→',
            Arrow::Down => '↓',
            Arrow::Left => '←',
        }
    }

    /// Rotates CW.
    fn rotate(self) -> Arrow {
        match self {
            Arrow::Up => Arrow::Right,
            Arrow::Right => Arrow::Down,
            Arrow::Down => Arrow::Left,
            Arrow::Left => Arrow::Up,
        }
    }

    /// How many CW rotations are needed to go from `self` to `other`.
    fn distance_to(self, other: Arrow) -> u8 {
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

#[derive(Debug)]
pub enum RowPokeError {
    OutOfBounds(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RowPoke {
    A,
    B,
    C,
    D,
}

impl TryFrom<u8> for RowPoke {
    type Error = RowPokeError;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(RowPoke::A),
            1 => Ok(RowPoke::B),
            2 => Ok(RowPoke::C),
            3 => Ok(RowPoke::D),
            _ => Err(RowPokeError::OutOfBounds(x)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(pub [Arrow; 4]);

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Row([a, b, c, d]) = self;
        writeln!(f, "┌────────────┐")?;
        writeln!(f, "│ {}  {}  {}  {} │", a, b, c, d)?;
        write!(f, "└────────────┘")
    }
}

impl Row {
    /// Whether all arrows are the same.
    fn aligned(&self) -> bool {
        let Row([a, b, c, d]) = self;
        a == b && b == c && c == d
    }

    pub fn poke(&self, p: RowPoke) -> Row {
        let Row([a, b, c, d]) = self;
        match p {
            RowPoke::A => Row([a.rotate(), b.rotate(), *c, *d]),
            RowPoke::B => Row([a.rotate(), b.rotate(), c.rotate(), *d]),
            RowPoke::C => Row([*a, b.rotate(), c.rotate(), d.rotate()]),
            RowPoke::D => Row([*a, *b, c.rotate(), d.rotate()]),
        }
    }

    fn poke_many(&self, ps: &[RowPoke]) -> Row {
        ps.iter().fold(self.clone(), |r, p| r.poke(*p))
    }

    fn poke_all(&self) -> [Row; 4] {
        [
            self.poke(RowPoke::A),
            self.poke(RowPoke::B),
            self.poke(RowPoke::C),
            self.poke(RowPoke::D),
        ]
    }

    /// Finds the shortest sequence of pokes needed to align the row.
    pub fn pokes_to_align(&self) -> Vec<RowPoke> {
        fn deduce_pokes(poke_count: u32, i: usize) -> Vec<RowPoke> {
            (0..poke_count)
                .map(|nth_poke| {
                    let x = i / 4usize.pow(nth_poke);
                    let x: u8 = (x % 4).try_into().unwrap();
                    x.try_into().unwrap()
                })
                .collect()
        }

        fn f(poke_count: u32, rows: Vec<Row>) -> Vec<RowPoke> {
            match rows.iter().enumerate().find(|(_, r)| r.aligned()) {
                Some((i, _)) => deduce_pokes(poke_count, i),
                None => f(
                    poke_count + 1,
                    rows.into_iter().flat_map(|r| r.poke_all()).collect(),
                ),
            }
        }

        f(0, vec![self.clone()])
    }
}
