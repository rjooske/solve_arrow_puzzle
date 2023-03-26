use itertools::Itertools;
use rayon::prelude::*;
use solve_arrow_puzzle::solve::{self, RowPoke};

fn main() {
    let all_arrows = [
        solve::Arrow::Up,
        solve::Arrow::Right,
        solve::Arrow::Down,
        solve::Arrow::Left,
    ];

    let all_rows: Vec<_> = [
        all_arrows.into_iter(),
        all_arrows.into_iter(),
        all_arrows.into_iter(),
        all_arrows.into_iter(),
    ]
    .into_iter()
    .multi_cartesian_product()
    .map(|x| solve::Row(x.try_into().unwrap()))
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
        let a = a.to_lut_index();
        let b = b.to_lut_index();
        a.cmp(&b)
    });

    let lut = all_solutions
        .into_iter()
        .map(|(_, pokes)| {
            pokes
                .iter()
                .map(|p| match p {
                    RowPoke::A => 'A',
                    RowPoke::B => 'B',
                    RowPoke::C => 'C',
                    RowPoke::D => 'D',
                })
                .collect::<String>()
        })
        .join(",");

    print!("{}", lut);
}
