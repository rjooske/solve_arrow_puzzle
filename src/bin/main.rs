use std::{process::exit, thread::sleep, time::Duration};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use device_query::{DeviceQuery, DeviceState};
use solve_arrow_puzzle::{
    app::play,
    device::{HeadlessDevice, Transform, Vec2},
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

fn main_play_expert() -> anyhow::Result<()> {
    let device = HeadlessDevice::new(
        1440,
        Transform::new(Vec2::new(721.0, 1094.0), Vec2::new(721.0, 2346.0)),
        Vec2::new(721.0, 2750.0),
    )
    .context("create headless device")?;
    play(device).context("play")?;
    Ok(())
}

fn main() {
    main_play_expert().unwrap();
}
