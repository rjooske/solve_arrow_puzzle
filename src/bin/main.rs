use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, MouseButton, MouseControllable};
use itertools::Itertools;
use scrap::{Capturer, Display};
use solve_arrow_puzzle::{
    android::Tapper,
    expert,
    gui::{ArrowToColor, Color, Dimensions, Point, Screen, ScreenView},
    solve::pokes_to_align_board,
};
use std::{
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};

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

fn watch<P>(screen: &mut Screen, timeout: Duration, predicate: P) -> Result<()>
where
    P: Fn(ScreenView) -> Result<bool>,
{
    let before = Instant::now();
    while before.elapsed() < timeout {
        if screen.view_and_map(|v| predicate(v))?? {
            break;
        }
        sleep(Duration::from_millis(1));
    }
    Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    fn add(self, other: Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn scale(self, s: f64) -> Vector {
        Vector {
            x: s * self.x,
            y: s * self.y,
        }
    }

    fn rotate(self, angle: f64) -> Vector {
        let sin = angle.sin();
        let cos = angle.cos();
        Vector {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    fn normalize(self) -> Vector {
        self.scale(1.0 / self.magnitude())
    }

    fn magnitude(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Transform {
    top_arrow: Vector,
    axis_a: Vector,
    axis_b: Vector,
}

impl Transform {
    fn new(top_arrow: Vector, bottom_arrow: Vector) -> Transform {
        use std::f64::consts::PI;

        let top_to_bottom = bottom_arrow.sub(top_arrow);
        let arrow_diameter = top_to_bottom.magnitude() / 6.0;
        let axis_a = top_to_bottom
            .rotate(-PI / 3.0)
            .normalize()
            .scale(arrow_diameter);
        let axis_b = top_to_bottom
            .rotate(PI / 3.0)
            .normalize()
            .scale(arrow_diameter);

        Transform {
            top_arrow,
            axis_a,
            axis_b,
        }
    }

    fn index_to_position(&self, x: usize, y: usize) -> Vector {
        self.axis_a
            .scale((x - 1) as f64)
            .add(self.axis_b.scale((y - 1) as f64))
            .add(self.top_arrow)
    }

    fn positions(&self) -> impl Iterator<Item = Vector> + '_ {
        expert::Board::POSITIONS
            .iter()
            .map(|&(x, y)| self.index_to_position(x as usize, y as usize))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnscreenArrow {
    Aligned,
    Unaligned(expert::Arrow),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OnscreenBoard {
    Aligned,
    Unaligned(expert::Board),
}

fn closest_onscreen_arrow(
    red_to_onscreen_arrow: &[(u8, OnscreenArrow)],
    target_red: u8,
) -> anyhow::Result<(u8, OnscreenArrow)> {
    red_to_onscreen_arrow
        .iter()
        .map(|&(red, arrow)| (red.abs_diff(target_red), arrow))
        .min_by_key(|&(red, _)| red)
        .context("empty red -> onscreen arrow mapping")
}

fn find_onscreen_board(
    red_to_onscreen_arrow: &[(u8, OnscreenArrow)],
    arrow_reds: &[u8],
    diff_threshold: u8,
) -> anyhow::Result<OnscreenBoard> {
    let diff_arrow_pairs = arrow_reds
        .iter()
        .map(|&red| closest_onscreen_arrow(red_to_onscreen_arrow, red))
        .collect::<anyhow::Result<Vec<_>>>()
        .context("map reds to onscreen arrows")?;

    if let Some(&pair) = diff_arrow_pairs
        .iter()
        .find(|&&(diff, _)| diff > diff_threshold)
    {
        return Err(anyhow!(
            "pair `{:?}` has diff above threshold `{}`: {:?}",
            pair,
            diff_threshold,
            diff_arrow_pairs
        ));
    }

    let unaligned_arrows = diff_arrow_pairs
        .iter()
        .filter_map(|&(_, a)| match a {
            OnscreenArrow::Aligned => None,
            OnscreenArrow::Unaligned(a) => Some(a),
        })
        .collect::<Vec<_>>();

    if !(unaligned_arrows.is_empty()
        || unaligned_arrows.len() == diff_arrow_pairs.len())
    {
        return Err(anyhow!(
            "board is neither aligned nor unaligned: {:?}",
            diff_arrow_pairs
        ));
    }

    if unaligned_arrows.is_empty() {
        Ok(OnscreenBoard::Aligned)
    } else {
        Ok(OnscreenBoard::Unaligned(expert::Board::from_arrows(
            unaligned_arrows.iter().copied(),
        )))
    }
}

fn main_play_expert() -> ! {
    let top = Vector { x: 225.0, y: 414.0 };
    let bottom = Vector { x: 228.0, y: 806.0 };
    let claim = Vector { x: 272.0, y: 939.0 };

    let red_to_onscreen_arrow: Vec<(u8, OnscreenArrow)> = vec![
        (27, OnscreenArrow::Aligned),
        (17, OnscreenArrow::Unaligned(expert::Arrow(0))),
        (30, OnscreenArrow::Unaligned(expert::Arrow(1))),
        (44, OnscreenArrow::Unaligned(expert::Arrow(2))),
        (57, OnscreenArrow::Unaligned(expert::Arrow(3))),
        (71, OnscreenArrow::Unaligned(expert::Arrow(4))),
        (85, OnscreenArrow::Unaligned(expert::Arrow(5))),
    ];
    // background red: 51

    let transform = Transform::new(top, bottom);

    let capturer = Capturer::new(Display::primary().unwrap()).unwrap();
    let mut screen = Screen::new(capturer);
    let device_state = DeviceState::new();
    let mut enigo = Enigo::new();

    let arrow_reds = screen
        .view_and_map(|view| {
            transform
                .positions()
                .map(|p| {
                    view.at_apple_silicon(p.x as usize, p.y as usize).unwrap().r
                })
                .collect::<Vec<_>>()
        })
        .unwrap();

    let board = find_onscreen_board(&red_to_onscreen_arrow, &arrow_reds, 2)
        .context("find onscreen board")
        .unwrap();
    let board = match board {
        OnscreenBoard::Aligned => panic!("aligned"),
        OnscreenBoard::Unaligned(b) => b,
    };

    println!("{}", board);

    let solve_pokes = board.solve().into_iter().dedup_with_count();
    let solve_clicks = solve_pokes.map(|(count, (x, y))| {
        (count, transform.index_to_position(x as usize, y as usize))
    });

    fn click(enigo: &mut Enigo, x: i32, y: i32, count: usize, delay: Duration) {
        enigo.mouse_move_to(x, y);
        sleep(delay);
        for _ in 0..count {
            enigo.mouse_down(MouseButton::Left);
            enigo.mouse_up(MouseButton::Left);
            sleep(delay);
        }
    }

    sleep(Duration::from_secs(3));

    for (count, p) in solve_clicks {
        click(
            &mut enigo,
            p.x as i32,
            p.y as i32,
            count,
            Duration::from_millis(1),
        );
    }

    exit(0);
}

fn main() -> Result<()> {
    let args = Args::parse();

    match (args.command, args.game_mode) {
        (Command::Setup, GameMode::Normal) => todo!(),
        (Command::Setup, GameMode::Expert) => main_setup_expert(),
        (Command::Play, GameMode::Normal) => todo!(),
        (Command::Play, GameMode::Expert) => main_play_expert(),
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
