use itertools::Itertools;
use rayon::prelude::*;
use solve_arrow_puzzle::{
    puzzle::{Arrow, Row, RowPoke},
    solve::row_lut_index,
};

fn main() {
    let all_arrows = [Arrow::Up, Arrow::Right, Arrow::Down, Arrow::Left];

    let all_rows: Vec<_> = [
        all_arrows.into_iter(),
        all_arrows.into_iter(),
        all_arrows.into_iter(),
        all_arrows.into_iter(),
    ]
    .into_iter()
    .multi_cartesian_product()
    .map(|x| Row(x.try_into().unwrap()))
    .collect();

    let mut all_solutions: Vec<_> = all_rows
        .into_par_iter()
        .map(|problem| {
            let mut solution = problem.pokes_to_align();
            solution.sort_unstable();
            (problem, solution)
        })
        .collect();

    all_solutions.sort_unstable_by(|(a, _), (b, _)| {
        let a = row_lut_index(a);
        let b = row_lut_index(b);
        a.cmp(&b)
    });

    let lut = all_solutions
        .into_iter()
        .map(|(_, pokes)| {
            let pokes = pokes
                .iter()
                .map(|p| {
                    let p = match p {
                        RowPoke::A => 'A',
                        RowPoke::B => 'B',
                        RowPoke::C => 'C',
                        RowPoke::D => 'D',
                    };
                    format!("RowPoke::{}", p)
                })
                .join(",");
            format!("&[{}]", pokes)
        })
        .join(",\n");

    print!(
        r#"
use crate::puzzle::RowPoke;

pub const SOLUTIONS: [&[RowPoke]; 256] = [
{}
];
        "#,
        lut
    );
}
