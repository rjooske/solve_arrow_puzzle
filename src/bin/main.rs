use itertools::{EitherOrBoth, Itertools};
use solve_arrow_puzzle::solve;

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
    let problem = solve::Row([
        solve::Arrow::Up,
        solve::Arrow::Left,
        solve::Arrow::Down,
        solve::Arrow::Right,
    ]);

    let pokes = problem.pokes_to_align_to(solve::Arrow::Up);

    let mut problem = problem;
    println!("{}", problem);
    for p in pokes {
        problem = problem.poke(p);
        println!("{}", problem);
    }
}
