mod enumerate_2d;
mod puzzle;

fn main() {
    let mut problem: puzzle::Board = "uldlrrulrldlrulu".parse().unwrap();

    let solution = puzzle::solve_bottom_3_rows(problem.clone());

    println!("{:?}", solution);
    println!("{}", problem);

    for m in solution.moves {
        problem = problem.poke(m.x, m.y);
        println!("{}", problem);
    }
}
