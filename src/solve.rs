use crate::{
    lut::SOLUTIONS,
    puzzle::{Arrow, Row, RowPoke},
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

pub fn pokes_to_align(r: &Row) -> &[RowPoke] {
    let i: usize = row_lut_index(r).into();
    SOLUTIONS[i]
}
