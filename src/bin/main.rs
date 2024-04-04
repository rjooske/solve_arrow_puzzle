use std::{process::exit, thread::sleep, time::Duration};

use clap::{Parser, ValueEnum};
use device_query::{DeviceQuery, DeviceState};
use solve_arrow_puzzle::{app::play, screen::HeadlessScreen};

#[derive(Debug, Parser)]
enum Command {
    Setup,
    Play,
}

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
enum GameMode {
    Normal,
    Expert,
}

#[derive(Debug, Parser)]
struct Args {
    // /// Path to whatever
    // #[arg(short, long)]
    // path: Option<PathBuf>,
    //
    // #[arg(short, long, default_value = "Hi!")]
    // greetings: String,
    //
    // /// What to do
    // #[command(subcommand)]
    // command: Command,
    /// What to do
    #[command(subcommand)]
    command: Command,

    /// Game mode
    #[arg(short, long)]
    game_mode: GameMode,
}

fn main_setup_expert() -> ! {
    fn held_alone(button_pressed: &[bool], button: usize) -> bool {
        button_pressed
            .iter()
            .enumerate()
            .skip(1)
            .all(|(i, &pressed)| if i == button { pressed } else { !pressed })
    }

    let state = DeviceState::new();

    while held_alone(&state.get_mouse().button_pressed, 1) {
        sleep(Duration::from_millis(10));
    }
    while !held_alone(&state.get_mouse().button_pressed, 1) {
        sleep(Duration::from_millis(10));
    }
    let top = state.get_mouse().coords;

    while held_alone(&state.get_mouse().button_pressed, 1) {
        sleep(Duration::from_millis(10));
    }
    while !held_alone(&state.get_mouse().button_pressed, 1) {
        sleep(Duration::from_millis(10));
    }
    let bottom = state.get_mouse().coords;

    while held_alone(&state.get_mouse().button_pressed, 1) {
        sleep(Duration::from_millis(10));
    }
    while !held_alone(&state.get_mouse().button_pressed, 1) {
        sleep(Duration::from_millis(10));
    }
    let claim = state.get_mouse().coords;

    println!("top: {:?}", top);
    println!("bottom: {:?}", bottom);
    println!("claim: {:?}", claim);

    exit(0);
}

// fn main_play_expert() -> ! {
//     // FIXME: don't hardcode these values
//     let top = Vector { x: 225.0, y: 414.0 };
//     let bottom = Vector { x: 228.0, y: 806.0 };
//     let claim = Vector { x: 272.0, y: 939.0 };
//     let red_to_onscreen_arrow: Vec<(u8, OnscreenArrow)> = vec![
//         (27, OnscreenArrow::Aligned),
//         (17, OnscreenArrow::Unaligned(expert::Arrow(0))),
//         (30, OnscreenArrow::Unaligned(expert::Arrow(1))),
//         (44, OnscreenArrow::Unaligned(expert::Arrow(2))),
//         (57, OnscreenArrow::Unaligned(expert::Arrow(3))),
//         (71, OnscreenArrow::Unaligned(expert::Arrow(4))),
//         (85, OnscreenArrow::Unaligned(expert::Arrow(5))),
//     ];
//     // background red: 51
//
//     let transform = Transform::new(top, bottom);
//     let mut player = Player::new(Instant::now());
//
//     let mut screen =
//         Screen::new(Capturer::new(Display::primary().unwrap()).unwrap());
//     let device_state = DeviceState::new();
//     let mut cursor = CursorController {
//         enigo: Enigo::new(),
//         wait: Duration::from_millis(1),
//     };
//
//     while device_state.get_keys().contains(&Keycode::Backspace) {
//         sleep(Duration::from_millis(1));
//     }
//     while !device_state.get_keys().contains(&Keycode::Backspace) {
//         sleep(Duration::from_millis(1));
//     }
//     sleep(Duration::from_secs(1));
//
//     while !device_state.get_keys().contains(&Keycode::Backspace) {
//         let arrow_reds = screen
//             .view_and_map(|view| {
//                 transform
//                     .positions()
//                     .map(|p| {
//                         match view.at_apple_silicon(p.x as usize, p.y as usize)
//                         {
//                             Some(c) => Ok(c.r),
//                             None => Err(anyhow!(
//                                 "({}, {}) is outside the screen",
//                                 p.x,
//                                 p.y
//                             )),
//                         }
//                     })
//                     .collect::<anyhow::Result<Vec<_>>>()
//             })
//             .context("view screen")
//             .unwrap()
//             .context("map screen view")
//             .unwrap();
//
//         let onscreen_board =
//             find_onscreen_board(&red_to_onscreen_arrow, &arrow_reds, 2)
//                 .context("find onscreen board")
//                 .unwrap();
//
//         let action = player.transition(PlayerTransitionContext {
//             now: Instant::now(),
//             onscreen_board: &onscreen_board,
//         });
//         let action = match action {
//             Ok(x) => x,
//             Err(err) => panic!("player transition: {}", err),
//         };
//         match action {
//             Action::Nothing => sleep(Duration::from_millis(10)),
//             Action::Solve(board) => {
//                 let solve_indices = board.clone().solve();
//                 let mut solve_clicks = Vec::<(usize, (u8, u8))>::with_capacity(
//                     solve_indices.len(),
//                 );
//                 for index in solve_indices {
//                     match solve_clicks.last_mut() {
//                         Some(last) if last.1 == index => last.0 += 1,
//                         _ => solve_clicks.push((1, index)),
//                     }
//                 }
//                 let solve_clicks = solve_clicks
//                     .into_iter()
//                     .map(|(count, (x, y))| {
//                         (
//                             count,
//                             transform.index_to_click_position(
//                                 x as usize, y as usize,
//                             ),
//                         )
//                     })
//                     .collect::<Vec<_>>();
//
//                 for (count, p) in solve_clicks {
//                     cursor.click_many(p.x as i32, p.y as i32, count);
//                 }
//             }
//             Action::ClaimRewards => {
//                 cursor.click(claim.x as i32, claim.y as i32)
//             }
//         }
//     }
//
//     exit(0);
// }

fn main_play_expert() -> anyhow::Result<()> {
    let screen = HeadlessScreen::new(472, 1024);
    sleep(Duration::from_secs(5));
    play(screen).unwrap();
    Ok(())
}

fn main() {
    let args = Args::parse();

    match (args.command, args.game_mode) {
        (Command::Setup, GameMode::Normal) => todo!(),
        (Command::Setup, GameMode::Expert) => main_setup_expert(),
        (Command::Play, GameMode::Normal) => todo!(),
        (Command::Play, GameMode::Expert) => main_play_expert().unwrap(),
    }
    todo!();

    // let dimensions: Dimensions =
    //     serde_json::from_str(&fs::read_to_string("dimensions.json")?)?;
    // let arrow_to_color: ArrowToColor =
    //     serde_json::from_str(&fs::read_to_string("arrow_to_color.json")?)?;
    //
    // let tapper_config = fs::read_to_string("tapper_config.json")?;
    // let tapper_config = serde_json::from_str(&tapper_config)?;
    // let mut tapper = Tapper::new(tapper_config)?;
    //
    // println!("press shift to begin");
    // let state = DeviceState::new();
    // while !state
    //     .get_keys()
    //     .into_iter()
    //     .any(|k| matches!(k, Keycode::LShift | Keycode::RShift))
    // {
    //     sleep(Duration::from_millis(10));
    // }
    //
    // let mut capturer = Capturer::new(Display::primary()?)?;
    //
    // println!("hold backspace to quit");
    // while !state.get_keys().contains(&Keycode::Backspace) {
    //     watch(&mut capturer, Duration::from_millis(500), |s| {
    //         let Point { x, y } = dimensions.first_arrow_position;
    //         let c = s.at(x as _, y as _)?;
    //         let d = c.euclidean_distance_to(Color {
    //             r: 27,
    //             g: 27,
    //             b: 27,
    //         });
    //         Ok(d > 3.0)
    //     })?;
    //
    //     let screenshot = ScreenView::take(&mut capturer)?;
    //     let board = detect_board(&dimensions, &arrow_to_color, &screenshot)?;
    //     let pokes = pokes_to_align_board(&board);
    //     tapper.tap_many(&pokes)?;
    //
    //     watch(&mut capturer, Duration::from_millis(500), |s| {
    //         let Point { x, y } = dimensions.first_arrow_position;
    //         let c = s.at(x as _, y as _)?;
    //         let d = c.euclidean_distance_to(Color {
    //             r: 27,
    //             g: 27,
    //             b: 27,
    //         });
    //         Ok(d < 3.0)
    //     })?;
    //     sleep(Duration::from_millis(50));
    //
    //     tapper.tap_claim_button()?;
    // }
    //
    // Ok(())
}
