use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use solve_arrow_puzzle::{
    config::Config,
    gui::{ArrowToColor, Color},
    puzzle::Arrow,
};
use std::{collections::HashMap, env, fs, io, thread::sleep, time::Duration};

use scrap::{Capturer, Display};

fn capture_frame(c: &mut Capturer) -> io::Result<Vec<u8>> {
    loop {
        match c.frame() {
            Ok(f) => return Ok(f.to_vec()),
            Err(err) => {
                if err.kind() == io::ErrorKind::WouldBlock {
                    sleep(Duration::from_millis(1));
                } else {
                    return Err(err);
                }
            }
        }
    }
}

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

    let config = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config)?;

    let display = Display::primary()?;
    let mut capturer = Capturer::new(display)?;
    let width = capturer.width();
    let height = capturer.height();

    let frame = capture_frame(&mut capturer)?;
    let frame: Vec<_> = frame
        .chunks(4)
        .take(width * height)
        .map(|x| match x {
            &[b, g, r, _] => Color { r, g, b },
            _ => unreachable!(),
        })
        .collect();

    let colors = (0..4)
        .cartesian_product(0..4)
        .map(|(y, x)| {
            let x = config.first_arrow_position.x + config.arrow_diameter * x;
            let y = config.first_arrow_position.y + config.arrow_diameter * y;
            let i: usize = x as usize + width * y as usize;
            match frame.get(i) {
                Some(c) => Ok(*c),
                None => Err(anyhow!(
                    "arrow position ({}, {}) landed outside the screen",
                    x,
                    y
                )),
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let arrow_to_color = parse_arrow_to_color(&arrows, &colors)?;
    println!("{}", serde_json::to_string(&arrow_to_color)?);

    Ok(())
}
