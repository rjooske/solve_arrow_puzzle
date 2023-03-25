use std::iter;

use itertools::{EitherOrBoth, Itertools};

mod enumerate_2d;
mod puzzle;

fn concat_linewise(a: &str, b: &str) -> String {
    a.lines()
        .zip_longest(b.lines())
        .map(|x| match x {
            EitherOrBoth::Both(a, b) => a.to_owned() + b,
            EitherOrBoth::Left(a) => a.to_owned(),
            EitherOrBoth::Right(b) => b.to_owned(),
        })
        .join("\n")
}

fn main() {
    let problem: puzzle::Board = "uldlrrulrldlrulu".parse().unwrap();

    let solver = puzzle::solve_nth_row(3)
        .and_then(|| puzzle::solve_nth_row(2))
        .and_then(|| puzzle::solve_nth_row(1));
    let puzzle::Solution(pokes) = solver.run(&problem);

    let intermediate_boards = pokes.iter().scan(problem.clone(), |b, p| {
        *b = b.poke(p);
        Some(b.clone())
    });

    let s = iter::once(problem)
        .chain(intermediate_boards)
        .zip_longest(pokes.iter())
        .map(|x| match x {
            EitherOrBoth::Both(b, p) => b.to_string_with_highlight(p),
            EitherOrBoth::Left(b) => b.to_string(),
            EitherOrBoth::Right(_) => unreachable!(),
        })
        .chunks(4)
        .into_iter()
        .map(|chunk| chunk.fold("".to_owned(), |acc, s| concat_linewise(&acc, &s)))
        .join("\n");
    println!("{}", s);
}
