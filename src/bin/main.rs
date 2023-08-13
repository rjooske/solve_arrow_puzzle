use anyhow::Result;
use clap::{Parser, ValueEnum};
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::Enigo;
use enigo::MouseButton;
use enigo::MouseControllable;
use itertools::Itertools;
use scrap::{Capturer, Display};
use solve_arrow_puzzle::{
    android::Tapper,
    expert,
    gui::{detect_board, ArrowToColor, Color, Dimensions, Point, Screenshot},
    solve::pokes_to_align_board,
};
use std::collections::HashMap;
use std::{
    fs,
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

    fn as_i32s(self) -> (i32, i32) {
        (self.x.round() as i32, self.y.round() as i32)
    }
}

fn main_play_expert() -> ! {
    use std::f64::consts::PI;

    let top = Vector { x: 225.0, y: 414.0 };
    let bottom = Vector { x: 228.0, y: 806.0 };
    let claim = Vector { x: 272.0, y: 939.0 };

    let arrow_diameter = bottom.sub(top).magnitude() / 6.0;
    let axis_a = bottom
        .sub(top)
        .rotate(-PI / 3.0)
        .normalize()
        .scale(arrow_diameter);
    let axis_b = bottom
        .sub(top)
        .rotate(PI / 3.0)
        .normalize()
        .scale(arrow_diameter);

    println!("axis_a: {:?}", axis_a);
    println!("axis_b: {:?}", axis_b);

    let mut capturer = Capturer::new(Display::primary().unwrap()).unwrap();
    let device_state = DeviceState::new();
    let mut enigo = Enigo::new();

    let arrow_positions = expert::Board::POSITIONS.iter().map(|&(x, y)| {
        axis_a
            .scale((x - 1) as f64)
            .add(axis_b.scale((y - 1) as f64))
            .add(top)
    });

    let screenshot = Screenshot::take(&mut capturer).unwrap();
    let mut arrow_reds = arrow_positions
        .map(|p| screenshot.at(p.x as usize, p.y as usize).unwrap())
        .map(|c| c.r)
        .collect_vec();

    let mut red_to_arrow = arrow_reds.clone();
    red_to_arrow.sort();
    red_to_arrow.dedup();

    if red_to_arrow.len() != 6 {
        panic!("insufficient arrow reds: {:?}", red_to_arrow);
    }

    let red_to_arrow = red_to_arrow
        .iter()
        .enumerate()
        .map(|(i, r)| (r, expert::Arrow(i.try_into().unwrap())))
        .collect::<HashMap<_, _>>();

    println!("{:?}", red_to_arrow);

    let arrows = arrow_reds.iter().map(|r| *red_to_arrow.get(r).unwrap());
    let board = expert::Board::from_arrows(arrows);
    println!("{}", board);

    let solve_pokes = board.solve().into_iter().dedup_with_count();
    let solve_clicks = solve_pokes.map(|(count, (x, y))| {
        let position = axis_a
            .scale((x - 1) as f64)
            .add(axis_b.scale((y - 1) as f64))
            .add(top);
        (count, position)
    });

    fn click(enigo: &mut Enigo, x: i32, y: i32, count: usize, delay: Duration) {
        enigo.mouse_move_to(x, y);
        sleep(delay);
        for _ in 0..count {
            enigo.mouse_down(MouseButton::Left);
            enigo.mouse_up(MouseButton::Left);
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
