use std::{
    fs,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, MouseButton, MouseControllable};
use scrap::{Capturer, Display};
use solve_arrow_puzzle::{
    gui::{detect_board, ArrowToColor, Color, Dimensions, Point, Screenshot},
    solve::pokes_to_align_board,
};

fn main() -> Result<()> {
    let dimensions: Dimensions =
        fs::read_to_string("dimensions.json")?.parse()?;
    let arrow_to_color: ArrowToColor =
        fs::read_to_string("arrow_to_color.json")?.parse()?;

    println!("press shift to begin");
    let state = DeviceState::new();
    while !state
        .get_keys()
        .into_iter()
        .any(|k| matches!(k, Keycode::LShift | Keycode::RShift))
    {
        sleep(Duration::from_millis(10));
    }

    let mut capturer = Capturer::new(Display::primary()?)?;
    let mut enigo = Enigo::new();

    let mut min_poke_count = usize::MAX;
    let mut max_poke_count = 0;

    println!("hold backspace to quit");
    while !state.get_keys().contains(&Keycode::Backspace) {
        let before = Instant::now();
        while before.elapsed() < Duration::from_millis(200) {
            if let Some(Color { r, g: _, b: _ }) =
                Screenshot::take(&mut capturer)?.at(
                    dimensions.first_arrow_position.x as _,
                    dimensions.first_arrow_position.y as _,
                )
            {
                if r != 27 {
                    break;
                }
            }
        }

        let screenshot = Screenshot::take(&mut capturer)?;
        let board = detect_board(&dimensions, &arrow_to_color, &screenshot)?;
        let pokes = pokes_to_align_board(&board);

        let len = pokes.len();
        if len < min_poke_count {
            min_poke_count = len;
            println!("min {}", len);
            println!("{}", board);
        }
        if len > max_poke_count {
            max_poke_count = len;
            println!("max {}", len);
            println!("{}", board);
        }

        for p in pokes {
            let Point { x, y } = dimensions.arrow_position(&p);
            click(&mut enigo, x as _, y as _, Duration::from_millis(1));
        }

        let before = Instant::now();
        while before.elapsed() < Duration::from_millis(200) {
            if let Some(Color { r, g, b }) = Screenshot::take(&mut capturer)?
                .at(
                    dimensions.first_arrow_position.x as _,
                    dimensions.first_arrow_position.y as _,
                )
            {
                if 27 == r && r == g && g == b {
                    break;
                }
            }
        }
        sleep(Duration::from_millis(40));

        click(
            &mut enigo,
            dimensions.claim_button_position.x as _,
            dimensions.claim_button_position.y as _,
            Duration::from_millis(1),
        );
    }

    Ok(())
}

fn click(enigo: &mut Enigo, x: i32, y: i32, delay: Duration) {
    enigo.mouse_move_to(x, y);
    sleep(delay);
    enigo.mouse_down(MouseButton::Left);
    sleep(delay);
    enigo.mouse_up(MouseButton::Left);
}
