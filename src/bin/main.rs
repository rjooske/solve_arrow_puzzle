use itertools::Itertools;
use solve_arrow_puzzle::{
    puzzle::{board, Arrow, Board, BoardPoke, Row},
    solve::pokes_to_align_board,
};

fn main() {
    let problem = board!(
    d d u u
    l r l u
    u u r u
    d u u r
    );

    let pokes = pokes_to_align_board(&problem);
    let len = pokes.len();

    let mut counts = [[0; 4]; 4];
    for BoardPoke(x, y) in pokes {
        let x: u8 = x.into();
        let y: u8 = y.into();
        let x: usize = x.into();
        let y: usize = y.into();
        counts[y][x] += 1;
    }

    let s: String = counts
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|n| {
                    let s: String = match n {
                        0 => "-".to_owned(),
                        n => n.to_string(),
                    };
                    format!(" {} ", s)
                })
                .collect::<String>()
        })
        .join("\n");

    println!("{}", s);
    println!("len={}", len);
}
