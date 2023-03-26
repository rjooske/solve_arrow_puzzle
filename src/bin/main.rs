use solve_arrow_puzzle::{
    puzzle::{Arrow, Row},
    solve::pokes_to_align,
};

fn main() {
    let problem = Row([Arrow::Up, Arrow::Left, Arrow::Down, Arrow::Right]);
    let pokes = pokes_to_align(&problem).to_owned();

    let mut problem = problem;
    println!("{}", problem);
    for p in pokes {
        problem = problem.poke(p);
        println!("{:?}\n{}", p, problem);
    }
}
