use std::{fs, thread::sleep, time::Duration};

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode};
use scrap::{Capturer, Display};
use solve_arrow_puzzle::{
    config::Config,
    gui::{ArrowToColor, Screenshot},
    puzzle::{Board, Row},
};

fn screenshot_to_board(
    ss: &Screenshot,
    cfg: &Config,
    atc: &ArrowToColor,
) -> Board {
    let rows = [0, 1, 2, 3].map(|y| {
        let arrows = [0, 1, 2, 3].map(|x| {
            let x = cfg.first_arrow_position.x + cfg.arrow_diameter * x;
            let y = cfg.first_arrow_position.y + cfg.arrow_diameter * y;
            let c = ss.at(x as usize, y as usize).expect("bad config");
            atc.closest(&c)
        });
        Row(arrows)
    });
    Board(rows)
}

fn main() -> Result<()> {
    // TODO: `Dimensions` or something
    let config: Config = fs::read_to_string("config.json")?.parse()?;
    let arrow_to_color: ArrowToColor =
        fs::read_to_string("arrow_to_color.json")?.parse()?;

    println!("press shift to begin");
    let state = DeviceState::new();
    loop {
        sleep(Duration::from_millis(10));
        let keys = state.get_keys();
        if keys
            .iter()
            .any(|k| matches!(k, Keycode::LShift | Keycode::RShift))
        {
            break;
        }
    }

    let mut capturer = Capturer::new(Display::primary()?)?;

    let screenshot = Screenshot::take(&mut capturer)?;
    let board = screenshot_to_board(&screenshot, &config, &arrow_to_color);

    println!("{}", board);

    Ok(())
}
