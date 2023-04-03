use std::{
    fs,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode};

use scrap::{Capturer, Display};
use solve_arrow_puzzle::{
    android::Tapper,
    gui::{detect_board, ArrowToColor, Color, Dimensions, Point, Screenshot},
    solve::pokes_to_align_board,
};

fn watch<P>(
    capturer: &mut Capturer,
    timeout: Duration,
    predicate: P,
) -> Result<()>
where
    P: Fn(Screenshot) -> Result<bool>,
{
    let before = Instant::now();
    while before.elapsed() < timeout {
        if predicate(Screenshot::take(capturer)?)? {
            break;
        }
        sleep(Duration::from_millis(1));
    }
    Ok(())
}

fn main() -> Result<()> {
    let dimensions: Dimensions =
        serde_json::from_str(&fs::read_to_string("dimensions.json")?)?;
    let arrow_to_color: ArrowToColor =
        serde_json::from_str(&fs::read_to_string("arrow_to_color.json")?)?;

    let tapper_config = fs::read_to_string("tapper_config.json")?;
    let tapper_config = serde_json::from_str(&tapper_config)?;
    let mut tapper = Tapper::new(tapper_config)?;

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

    println!("hold backspace to quit");
    while !state.get_keys().contains(&Keycode::Backspace) {
        watch(&mut capturer, Duration::from_millis(500), |s| {
            let Point { x, y } = dimensions.first_arrow_position;
            let c = s.at(x as _, y as _)?;
            let d = c.euclidean_distance_to(Color {
                r: 27,
                g: 27,
                b: 27,
            });
            Ok(d > 3.0)
        })?;

        let screenshot = Screenshot::take(&mut capturer)?;
        let board = detect_board(&dimensions, &arrow_to_color, &screenshot)?;
        let pokes = pokes_to_align_board(&board);
        tapper.tap_many(&pokes)?;

        watch(&mut capturer, Duration::from_millis(500), |s| {
            let Point { x, y } = dimensions.first_arrow_position;
            let c = s.at(x as _, y as _)?;
            let d = c.euclidean_distance_to(Color {
                r: 27,
                g: 27,
                b: 27,
            });
            Ok(d < 3.0)
        })?;
        sleep(Duration::from_millis(50));

        tapper.tap_claim_button()?;
    }

    Ok(())
}
