use std::fmt::{Display, Write};

#[derive(PartialEq, Eq, Clone, Debug)]
enum Arrow {
    Up,
    Right,
    Down,
    Left,
}

impl Arrow {
    fn cw(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    fn ccw(&self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Right => Self::Down,
            Self::Down => Self::Right,
            Self::Left => Self::Up,
        }
    }
}

impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Up => '↑',
            Self::Right => '→',
            Self::Down => '↓',
            Self::Left => '←',
        };
        f.write_char(c)
    }
}

struct Board {
    arrows: [[Arrow; 3]; 3],
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─────────┐")?;
        for [a, b, c] in &self.arrows {
            writeln!(f, "│ {}  {}  {} │", a, b, c)?;
        }
        write!(f, "└─────────┘")
    }
}

impl Board {
    fn poke(&self, x: usize, y: usize) -> Self {
        let arrows: [[Arrow; 3]; 3] = self
            .arrows
            .iter()
            .enumerate()
            .map(|(ay, arrows)| {
                arrows
                    .iter()
                    .enumerate()
                    .map(|(ax, arrow)| {
                        let dx = x as i64 - ax as i64;
                        let dy = y as i64 - ay as i64;
                        match (dx, dy) {
                            (-1..=1, -1..=1) => arrow.cw(),
                            _ => arrow.clone(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("")
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("");
        Self { arrows }
    }
}

fn main() {
    let b = Board {
        arrows: [
            [Arrow::Up, Arrow::Up, Arrow::Up],
            [Arrow::Up, Arrow::Up, Arrow::Up],
            [Arrow::Up, Arrow::Up, Arrow::Up],
        ],
    };
    println!("{}", b);
    let b = b.poke(0, 0);
    println!("{}", b);
}
