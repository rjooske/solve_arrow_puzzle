use rayon::prelude::*;
use std::{
    fmt::{Display, Write},
    io,
    str::FromStr,
};

#[derive(PartialEq, Eq, Clone, Debug)]
enum Arrow {
    Up,
    Right,
    Down,
    Left,
}

impl Arrow {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'u' => Some(Arrow::Up),
            'r' => Some(Arrow::Right),
            'd' => Some(Arrow::Down),
            'l' => Some(Arrow::Left),
            _ => None,
        }
    }

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

#[derive(Debug, PartialEq, Eq, Clone)]
struct Board {
    arrows: [Arrow; 9],
}

impl FromStr for Board {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arrows: Option<[Arrow; 9]> = s
            .chars()
            .take(9)
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
        // self.arrows.windows(2).into_iter().all(|x| match x {
        //     [a, b] => a == b,
        //     _ => unreachable!("must match the arm above because of `windows(2)`"),
        // })
        let mut v = self.arrows.to_vec();
        v.dedup();
        matches!(
            v.as_slice(),
            [Arrow::Up, Arrow::Down]
                | [Arrow::Down, Arrow::Up]
                | [Arrow::Left, Arrow::Right]
                | [Arrow::Right, Arrow::Left]
        )
    }

    fn poke(&self, x: u8, y: u8) -> Self {
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

fn next_possible_boards((b, m): &(Board, Moves)) -> [(Board, Moves); 8] {
    [0u8, 1, 2, 3, 5, 6, 7, 8].map(|i| {
        let x = i % 3;
        let y = i / 3;
        let mut m = m.clone();
        m.push((x, y));
        (b.poke(x, y), m)
    })
}

type Moves = Vec<(u8, u8)>;

fn main() {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("cannot read stdin");

    // let problem: Board = "uuddlrlru".parse().expect("oops");
    let problem: Board = buf.parse().expect("oops");

    let mut boards = vec![(problem.clone(), vec![])];
    for i in 0.. {
        // match boards.par_iter().find_any(|&(b, _)| b.aligned()) {
        match boards.par_iter().find_any(|&(b, _)| b.aligned()) {
            Some((_, m)) => {
                println!("solution found!");
                println!("{:?}", m);
                let mut p = problem;
                for &(x, y) in m {
                    println!("{}", p);
                    p = p.poke(x, y);
                }
                println!("{}", p);
                break;
            }
            None => println!("attempt #{}", i),
        };
        println!("{} boards", boards.len());
        boards = boards.par_iter().flat_map(next_possible_boards).collect();
    }
}

// (1, 2), (1, 2), (2, 2), (2, 2),
// (1, 1), (1, 1), (2, 1), (2, 1)
