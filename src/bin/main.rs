use std::iter;

use itertools::{EitherOrBoth, Itertools};
use solve_arrow_puzzle::{
    puzzle::{Arrow, Board, Row},
    solve::pokes_to_align_board,
};

macro_rules! board {
    (
        $a:ident $b:ident $c:ident $d:ident
        $e:ident $f:ident $g:ident $h:ident
        $i:ident $j:ident $k:ident $l:ident
        $m:ident $n:ident $o:ident $p:ident
    ) => {{
        macro_rules! arrow {
            (u) => {
                Arrow::Up
            };
            (r) => {
                Arrow::Right
            };
            (d) => {
                Arrow::Down
            };
            (l) => {
                Arrow::Left
            };
        }
        Board([
            Row([arrow!($a), arrow!($b), arrow!($c), arrow!($d)]),
            Row([arrow!($e), arrow!($f), arrow!($g), arrow!($h)]),
            Row([arrow!($i), arrow!($j), arrow!($k), arrow!($l)]),
            Row([arrow!($m), arrow!($n), arrow!($o), arrow!($p)]),
        ])
    }};
}

fn concat_linewise(a: &str, b: &str) -> String {
    a.split('\n')
        .zip_longest(b.split('\n'))
        .map(|x| match x {
            EitherOrBoth::Both(a, b) => a.to_owned() + b,
            EitherOrBoth::Left(a) => a.to_owned(),
            EitherOrBoth::Right(b) => b.to_owned(),
        })
        .join("\n")
}

fn main() {
    let problem = board!(
        u l l d
        u r l d
        u d u u
        d l u u
    );

    let pokes = pokes_to_align_board(&problem);
    let len = pokes.len();

    let s = pokes
        .iter()
        .scan(problem.clone(), |b, p| {
            let tmp = b.to_string_with_highlight(*p);
            *b = b.poke(*p);
            Some(tmp)
        })
        .chain(iter::once(problem.poke_many(&pokes).to_string()))
        .chunks(4)
        .into_iter()
        .map(|it| it.fold(String::new(), |acc, s| concat_linewise(&acc, &s)))
        .join("\n");

    println!("{}", s);
    println!("len={}", len);
}
