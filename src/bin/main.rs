use anyhow::Context;
use clap::{Parser, ValueEnum};
use solve_arrow_puzzle::{
    app::play,
    device::{HeadlessDevice, OnscreenDevice, Transform, Vec2},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    Onscreen,
    Headless,
}

#[derive(Debug, Parser)]
struct Args {
    mode: Mode,
}

fn run(args: Args) -> anyhow::Result<()> {
    match args.mode {
        Mode::Onscreen => play(
            OnscreenDevice::new(
                Transform::new(Vec2::new(202.0, 437.0), Vec2::new(202.0, 831.0)),
                Vec2::new(203.0, 934.0),
            )
            .context("create onscreen device")?,
        ),
        Mode::Headless => play(
            HeadlessDevice::new(
                1440,
                Transform::new(Vec2::new(721.0, 1094.0), Vec2::new(721.0, 2346.0)),
                Vec2::new(721.0, 2750.0),
            )
            .context("create headless device")?,
        ),
    }
    .context("play")
}

fn main() {
    run(Args::parse()).unwrap();
}
