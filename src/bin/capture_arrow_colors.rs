use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use solve_arrow_puzzle::{
    gui::{ArrowToColor, Color, Dimensions, Point, Screen},
    puzzle::{Arrow, BoardPoke, RowPoke},
};
use std::{collections::HashMap, env, fs};

use scrap::{Capturer, Display};

fn parse_arg(arg: &str) -> Result<Vec<Arrow>> {
    arg.chars()
        .take(16)
        .map(|c| match c {
            'u' => Ok(Arrow::Up),
            'r' => Ok(Arrow::Right),
            'd' => Ok(Arrow::Down),
            'l' => Ok(Arrow::Left),
            c => Err(anyhow!("want `u`, `r`, `d`, or `l`, got `{}`", c)),
        })
        .collect::<Result<Vec<_>>>()
}

fn parse_arrow_to_color(arrows: &[Arrow], colors: &[Color]) -> Result<ArrowToColor> {
    fn f(map: &HashMap<&Arrow, Vec<&Color>>, a: &Arrow) -> Result<Color> {
        let c = **map
            .get(a)
            .with_context(|| format!("did not get color for `{}`", a))?
            .first()
            .with_context(|| format!("did not get color for `{}`", a))?;
        Ok(c)
    }

    let map = arrows.iter().zip(colors.iter()).into_group_map();
    let up = f(&map, &Arrow::Up)?;
    let right = f(&map, &Arrow::Right)?;
    let down = f(&map, &Arrow::Down)?;
    let left = f(&map, &Arrow::Left)?;

    Ok(ArrowToColor {
        up,
        right,
        down,
        left,
    })
}

fn main() -> Result<()> {
    let [_, arg]: [String; 2] = env::args().collect::<Vec<_>>().try_into().unwrap();
    let arrows = parse_arg(&arg)?;

    let dimensions: Dimensions = fs::read_to_string("dimensions.json")?.parse()?;

    let capturer = Capturer::new(Display::primary()?)?;
    let screenshot = Screen::new(capturer).view_and_map(|s| s.to_buf()).unwrap();

    let pokes = [RowPoke::A, RowPoke::B, RowPoke::C, RowPoke::D];
    let colors = pokes
        .into_iter()
        .cartesian_product(pokes.into_iter())
        .map(|(y, x)| {
            let Point { x, y } = dimensions.arrow_position(&BoardPoke(x, y));
            screenshot
                .as_view()
                .at_apple_silicon(x as _, y as _)
                .with_context(|| format!("({}, {}) is outside the screen", x, y))
        })
        .collect::<Result<Vec<_>>>()?;

    let arrow_to_color = parse_arrow_to_color(&arrows, &colors)?;
    println!("{}", serde_json::to_string_pretty(&arrow_to_color)?);

    Ok(())
}
