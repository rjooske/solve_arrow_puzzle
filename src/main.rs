use anyhow::Context;
use clap::Parser;
use solve_arrow_puzzle::{
    app::play,
    device::{ScrcpyDevice, Transform, Vec2},
};

#[derive(Debug, Parser)]
struct Args {}

fn run(args: Args) -> anyhow::Result<()> {
    let device = ScrcpyDevice::new(
        1440,
        3120,
        Vec2::new(721.0, 2750.0),
        2,
        Transform::new(Vec2::new(721.5, 1178.5), Vec2::new(721.5, 2430.5)),
        10001,
        10002,
    )
    .context("create scrcpy server device")?;
    play(device).context("play")
}

fn main() {
    run(Args::parse()).unwrap();
}
