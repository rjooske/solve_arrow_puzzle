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

#[derive(Debug, PartialEq, Eq)]
struct Board {
    arrows: [Arrow; 9],
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn row(f: &mut std::fmt::Formatter<'_>, a: &[Arrow]) -> std::fmt::Result {
            writeln!(f, "│ {}  {}  {} │", a[0], a[1], a[2])
        }
        writeln!(f, "┌─────────┐")?;
        row(f, &self.arrows[0..3])?;
        row(f, &self.arrows[3..6])?;
        row(f, &self.arrows[6..9])?;
        write!(f, "└─────────┘")
    }
}

impl Board {
    fn aligned(&self) -> bool {
        self.arrows.windows(2).into_iter().all(|x| match x {
            [a, b] => a == b,
            _ => unreachable!("must match the arm above because of `windows(2)`"),
        })
    }

    fn poke(&self, x: usize, y: usize) -> Self {
        let arrows: [Arrow; 9] = self
            .arrows
            .iter()
            .enumerate()
            .map(|(i, arrow)| {
                let dx = x as i64 - (i % 3) as i64;
                let dy = y as i64 - (i / 3) as i64;
                match (dx, dy) {
                    (-1..=1, -1..=1) => arrow.cw(),
                    _ => arrow.clone(),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("want exactly 9 elements");
        Self { arrows }
    }
}

fn main() {
    let answer = Board {
        arrows: [
            Arrow::Up,
            Arrow::Up,
            Arrow::Up,
            Arrow::Up,
            Arrow::Up,
            Arrow::Up,
            Arrow::Up,
            Arrow::Up,
            Arrow::Up,
        ],
    };
    let mut problem = Board {
        arrows: [
            Arrow::Down,
            Arrow::Up,
            Arrow::Right,
            Arrow::Up,
            Arrow::Up,
            Arrow::Right,
            Arrow::Left,
            Arrow::Left,
            Arrow::Right,
        ],
    };

    let mut moves = Vec::new();
    while answer != problem {
        let x = rand::random::<usize>() % 3;
        let y = rand::random::<usize>() % 3;
        problem = problem.poke(x, y);
        moves.push((x, y));
        // println!("{}", problem)
    }
    println!("{} moves", moves.len());
}
