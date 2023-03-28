use std::{fs, thread::sleep, time::Duration};

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, MouseButton, MouseControllable};
use mouse_rs::{types::keys::Keys, Mouse};
use scrap::{Capturer, Display};
use solve_arrow_puzzle::{
    config::Config,
    gui::{ArrowToColor, Point, Screenshot},
    puzzle::{Board, Row},
    solve::pokes_to_align_board,
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
    // let mouse = Mouse::new();
    // println!("start");
    // sleep(Duration::from_millis(5000));
    // println!("press");
    // mouse.press(&Keys::LEFT).unwrap();
    // sleep(Duration::from_millis(500));
    // println!("release");
    // mouse.release(&Keys::LEFT).unwrap();
    // return Ok(());

    // let mut enigo = Enigo::new();
    // println!("start");
    // sleep(Duration::from_millis(5000));
    // println!("down");
    // enigo.mouse_down(MouseButton::Left);
    // sleep(Duration::from_millis(500));
    // println!("up");
    // enigo.mouse_up(MouseButton::Left);

    // TODO: `Dimensions` or something
    let config: Config = fs::read_to_string("config.json")?.parse()?;
    let arrow_to_color: ArrowToColor =
        fs::read_to_string("arrow_to_color.json")?.parse()?;

    let state = DeviceState::new();
    println!("press shift to begin");
    loop {
        sleep(Duration::from_millis(10));
        let keys = state.get_keys();
        if keys
            .iter()
            .any(|k| matches!(k, Keycode::LShift | Keycode::RShift))
        {
            break;
        } else if keys.contains(&Keycode::Backspace) {
            return Ok(());
        }
    }

    let mut capturer = Capturer::new(Display::primary()?)?;

    while !state.get_keys().contains(&Keycode::Backspace) {
        sleep(Duration::from_millis(150));

        let screenshot = Screenshot::take(&mut capturer)?;
        let board = screenshot_to_board(&screenshot, &config, &arrow_to_color);
        let pokes = pokes_to_align_board(&board);
        println!("{} pokes", pokes.len());

        let mut enigo = Enigo::new();
        for p in pokes {
            let Point { x, y } = config.arrow_position(&p);
            click(&mut enigo, x as _, y as _, Duration::from_millis(1));
        }
        sleep(Duration::from_millis(150));
        click(
            &mut enigo,
            config.claim_button_position.x as _,
            config.claim_button_position.y as _,
            Duration::from_millis(1),
        );
    }

    Ok(())

    // Ok(())
}

fn click(enigo: &mut Enigo, x: i32, y: i32, delay: Duration) {
    enigo.mouse_move_to(x, y);
    sleep(delay);
    enigo.mouse_down(MouseButton::Left);
    sleep(delay);
    enigo.mouse_up(MouseButton::Left);
}
