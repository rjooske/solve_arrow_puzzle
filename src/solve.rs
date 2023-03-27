use std::collections::HashMap;

use crate::{
    lut::SOLUTIONS,
    puzzle::{Arrow, Board, BoardPoke, Row, RowPoke},
};

fn arrow_lut_index(a: &Arrow) -> u8 {
    match a {
        Arrow::Up => 0,
        Arrow::Right => 1,
        Arrow::Down => 2,
        Arrow::Left => 3,
    }
}

pub fn row_lut_index(r: &Row) -> u8 {
    let Row([a, b, c, d]) = r;
    let a = arrow_lut_index(a);
    let b = arrow_lut_index(b);
    let c = arrow_lut_index(c);
    let d = arrow_lut_index(d);
    a + 4 * b + 16 * c + 64 * d
}

fn pokes_to_align(r: &Row) -> &[RowPoke] {
    let i: usize = row_lut_index(r).into();
    SOLUTIONS[i]
}

fn board_pokes_at_nth_row(n: RowPoke, ps: &[RowPoke]) -> Vec<BoardPoke> {
    ps.iter().map(|p| BoardPoke(*p, n)).collect()
}

fn first_column_as_row(b: &Board) -> Row {
    let Board([r1, r2, r3, r4]) = b;
    let Row([a, _, _, _]) = r1;
    let Row([b, _, _, _]) = r2;
    let Row([c, _, _, _]) = r3;
    let Row([d, _, _, _]) = r4;
    Row([*a, *b, *c, *d])
}

fn remove_unnecessary_pokes(ps: &[BoardPoke]) -> Vec<BoardPoke> {
    let mut poke_to_counts = HashMap::new();
    for p in ps {
        poke_to_counts.entry(p).and_modify(|n| *n += 1).or_insert(1);
    }

    poke_to_counts
        .into_iter()
        .flat_map(|(p, n)| [*p].repeat(n % 4))
        .collect()
}

pub fn pokes_to_align_board(board: &Board) -> Vec<BoardPoke> {
    let mut board_pokes = Vec::new();

    let Board([row_a, _, _, _]) = board;
    let mut pokes = board_pokes_at_nth_row(RowPoke::B, pokes_to_align(row_a));
    let board = board.poke_many(&pokes);
    board_pokes.append(&mut pokes);

    let Board([_, row_b, _, _]) = &board;
    let mut pokes = board_pokes_at_nth_row(RowPoke::C, pokes_to_align(row_b));
    let board = board.poke_many(&pokes);
    board_pokes.append(&mut pokes);

    let Board([_, _, row_c, _]) = &board;
    let mut pokes = board_pokes_at_nth_row(RowPoke::D, pokes_to_align(row_c));
    let board = board.poke_many(&pokes);
    board_pokes.append(&mut pokes);

    let Board([_, _, _, row_d]) = &board;
    let mut pokes = pokes_to_align(row_d)
        .iter()
        .flat_map(|&p| {
            [
                BoardPoke(p, RowPoke::D),
                BoardPoke(p, RowPoke::A),
                BoardPoke(p, RowPoke::B),
                BoardPoke(p, RowPoke::B),
                BoardPoke(p, RowPoke::B),
            ]
        })
        .collect::<Vec<_>>();
    let board = board.poke_many(&pokes);
    board_pokes.append(&mut pokes);

    let column = first_column_as_row(&board);
    let mut pokes = pokes_to_align(&column)
        .iter()
        .flat_map(|&p| [BoardPoke(RowPoke::A, p), BoardPoke(RowPoke::D, p)])
        .collect::<Vec<_>>();
    let board = board.poke_many(&pokes);
    board_pokes.append(&mut pokes);

    let mut pokes = [
        BoardPoke(RowPoke::A, RowPoke::A),
        BoardPoke(RowPoke::D, RowPoke::A),
        BoardPoke(RowPoke::A, RowPoke::D),
        BoardPoke(RowPoke::D, RowPoke::D),
    ]
    .repeat(board.0[0].0[0].distance_to(Arrow::Up).into());
    board_pokes.append(&mut pokes);

    board_pokes = remove_unnecessary_pokes(&board_pokes);
    board_pokes.sort_unstable();

    board_pokes
}
