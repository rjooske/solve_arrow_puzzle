use std::{
    fmt::Write as _,
    io::{Read, Write as _},
    process::{ChildStdin, Command, Stdio},
};

use anyhow::Context;
use flate2::read::GzDecoder;
use itertools::Itertools;
use phf::phf_map;

use crate::{
    app::Device,
    expert::{Arrow, Board},
    hex::Hex,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn scale(self, s: f64) -> Vec2 {
        Vec2 {
            x: s * self.x,
            y: s * self.y,
        }
    }

    fn rotate(self, angle: f64) -> Vec2 {
        let sin = angle.sin();
        let cos = angle.cos();
        Vec2 {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    fn normalize(self) -> Vec2 {
        self.scale(1.0 / self.magnitude())
    }

    fn magnitude(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn round_as_i32(self) -> (i32, i32) {
        (self.x.round() as i32, self.y.round() as i32)
    }

    fn round_as_usize(self) -> (usize, usize) {
        (self.x.round() as usize, self.y.round() as usize)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    top_arrow: Vec2,
    arrow_diameter: f64,
    axis_a: Vec2,
    axis_b: Vec2,
}

impl Transform {
    pub fn new(top_arrow: Vec2, bottom_arrow: Vec2) -> Transform {
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
            arrow_diameter,
            axis_a,
            axis_b,
        }
    }

    fn index_to_position(&self, x: usize, y: usize) -> Vec2 {
        self.axis_a
            .scale(x as f64)
            .add(self.axis_b.scale(y as f64))
            .add(self.top_arrow)
    }
}

#[derive(Debug)]
/// FIXME: currently doesn't clean up anything
pub struct HeadlessDevice {
    width: usize,
    claim_button: Vec2,
    arrow_positions: Hex<(usize, usize)>,
    screencap_output: Vec<u8>,
    adb_shell_input_stdin: ChildStdin,
}

static RED_TO_ARROW: phf::Map<u8, Arrow> = phf_map! {
    27u8 => Arrow(0),
    17u8 => Arrow(0),
    30u8 => Arrow(1),
    44u8 => Arrow(2),
    57u8 => Arrow(3),
    71u8 => Arrow(4),
    85u8 => Arrow(5),
};

impl Device for HeadlessDevice {
    fn detect_board(&mut self) -> anyhow::Result<Board> {
        let output = Command::new("adb")
            .arg("shell")
            .arg("screencap | gzip -c -k -1 /dev/stdin")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        self.screencap_output.clear();
        GzDecoder::new(output.stdout.as_slice())
            .read_to_end(&mut self.screencap_output)
            .unwrap();

        let rgbas = self
            .screencap_output
            .get(16..)
            .context("screencap output too small")?;
        let arrows: Vec<_> = self
            .arrow_positions
            .enumerate()
            .map(|(_, _, &(x, y))| {
                let r = rgbas[4 * (x + self.width * y)];
                RED_TO_ARROW
                    .get(&r)
                    .copied()
                    .with_context(|| format!("no arrows correspond to red value {}", r))
            })
            .try_collect()?;
        Ok(Board::from_arrows(arrows))
    }

    fn tap_board(&mut self, taps: Hex<u8>) -> anyhow::Result<()> {
        let mut commands = String::new();
        for ((_, _, &n), (_, _, &(x, y))) in taps.enumerate().zip(self.arrow_positions.enumerate())
        {
            for _ in 0..n {
                writeln!(commands, "input tap {} {} &", x, y).unwrap();
            }
        }
        self.adb_shell_input_stdin
            .write_all(commands.as_bytes())
            .context("write to adb shell stdin for input")?;
        Ok(())
    }

    fn tap_claim_button(&mut self) -> anyhow::Result<()> {
        writeln!(
            self.adb_shell_input_stdin,
            "input tap {} {}",
            self.claim_button.x, self.claim_button.y
        )
        .context("write to adb shell stdin for input")?;
        Ok(())
    }
}

impl HeadlessDevice {
    pub fn new(
        width: usize,
        transform: Transform,
        claim_button: Vec2,
    ) -> anyhow::Result<HeadlessDevice> {
        let arrow_positions =
            Hex::from_fn(|x, y| transform.index_to_position(x, y).round_as_usize());

        let mut adb_shell_input = Command::new("adb")
            .arg("shell")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn adb shell for input")?;

        Ok(HeadlessDevice {
            width,
            claim_button,
            arrow_positions,
            screencap_output: Vec::new(),
            adb_shell_input_stdin: adb_shell_input
                .stdin
                .take()
                .context("take adb shell stdin for input")?,
        })
    }
}
