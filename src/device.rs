use std::{
    fmt::Write as _,
    io::{Read as _, Write as _},
    process::{ChildStdin, ChildStdout, Command, Stdio},
};

use anyhow::Context;
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

    fn index_to_position(&self, x: u8, y: u8) -> Vec2 {
        self.axis_a
            .scale((x - 1) as f64)
            .add(self.axis_b.scale((y - 1) as f64))
            .add(self.top_arrow)
    }
}

#[derive(Debug)]
/// FIXME: currently doesn't clean up anything
pub struct HeadlessDevice {
    transform: Transform,
    claim_button: Vec2,
    rgbas: Vec<u8>,
    adb_shell_input_stdin: ChildStdin,
    adb_shell_screencap_stdin: ChildStdin,
    adb_shell_screencap_stdout: ChildStdout,
}

// background: 51

static RED_TO_ARROW: phf::Map<u8, Arrow> = phf_map! {
    27u8 => Arrow(0),
    17u8 => Arrow(0),
    30u8 => Arrow(1),
    44u8 => Arrow(2),
    57u8 => Arrow(3),
    71u8 => Arrow(4),
    85u8 => Arrow(5),
};

// FIXME: don't hardcode these values
// onscreen
// let top = Vec2 { x: 225.0, y: 414.0 };
// let bottom = Vec2 { x: 228.0, y: 806.0 };
// let claim = Vec2 { x: 272.0, y: 939.0 };
// let red_to_onscreen_arrow: Vec<(u8, OnscreenArrow)> = vec![
//     (27, OnscreenArrow::Aligned),
//     (17, OnscreenArrow::Unaligned(Arrow(0))),
//     (30, OnscreenArrow::Unaligned(Arrow(1))),
//     (44, OnscreenArrow::Unaligned(Arrow(2))),
//     (57, OnscreenArrow::Unaligned(Arrow(3))),
//     (71, OnscreenArrow::Unaligned(Arrow(4))),
//     (85, OnscreenArrow::Unaligned(Arrow(5))),
// ];
// background red: 51

// headless
// let top = Vec2 { x: 236.0, y: 357.0 };
// let bottom = Vec2 { x: 236.0, y: 767.0 };
// let claim = Vec2 { x: 236.0, y: 904.0 };
// let red_to_onscreen_arrow: Vec<(u8, OnscreenArrow)> = vec![
//     (25, OnscreenArrow::Aligned),
//     (16, OnscreenArrow::Unaligned(Arrow(0))),
//     (28, OnscreenArrow::Unaligned(Arrow(1))),
//     (42, OnscreenArrow::Unaligned(Arrow(2))),
//     (55, OnscreenArrow::Unaligned(Arrow(3))),
//     (69, OnscreenArrow::Unaligned(Arrow(4))),
//     (83, OnscreenArrow::Unaligned(Arrow(5))),
// ];
// background red: 49

impl Device for HeadlessDevice {
    type DetectBoardError = anyhow::Error;
    type TapBoardError = anyhow::Error;
    type TapClaimButtonError = anyhow::Error;

    fn detect_board(&mut self) -> Result<Board, Self::DetectBoardError> {
        self.adb_shell_screencap_stdin
            .write_all(b"screencap\n")
            .context("write to adb shell stdin for screencap")?;
        let mut width = [0u8; 4];
        let mut height = [0u8; 4];
        let mut pixel_format = [0u8; 4];
        self.adb_shell_screencap_stdout
            .read_exact(&mut width)
            .context("read screen width")?;
        self.adb_shell_screencap_stdout
            .read_exact(&mut height)
            .context("read screen height")?;
        self.adb_shell_screencap_stdout
            .read_exact(&mut pixel_format)
            .context("read pixel format")?;
        self.adb_shell_screencap_stdout
            .read_exact(&mut [0; 4])
            .context("read padding before rgba")?;
        let width = u32::from_le_bytes(width) as usize;
        let height = u32::from_le_bytes(height) as usize;
        self.rgbas.resize(4 * width * height, 0);
        self.adb_shell_screencap_stdout
            .read_exact(&mut self.rgbas)
            .context("read rgba")?;
        let arrows: Vec<_> = Board::POSITIONS
            .into_iter()
            .map(|(x, y)| {
                let (x, y) = self.transform.index_to_position(x, y).round_as_usize();
                let r = self.rgbas[4 * (x + width * y)];
                RED_TO_ARROW
                    .get(&r)
                    .copied()
                    .with_context(|| format!("no arrows correspond to red value {}", r))
            })
            .try_collect()?;
        Ok(Board::from_arrows(arrows))
    }

    fn tap_board(&mut self, taps: Hex<u8>) -> Result<(), Self::TapBoardError> {
        let mut commands = String::new();
        for (x, y, &n) in taps.enumerate() {
            let x = x as u8 + 1;
            let y = y as u8 + 1;
            let p = self.transform.index_to_position(x, y);
            for _ in 0..n {
                writeln!(commands, "input tap {} {} &", p.x, p.y).unwrap();
            }
        }
        writeln!(commands, "wait").unwrap();
        self.adb_shell_input_stdin
            .write_all(commands.as_bytes())
            .context("write to adb shell stdin for input")?;
        Ok(())
    }

    fn tap_claim_button(&mut self) -> Result<(), Self::TapClaimButtonError> {
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
    pub fn new(transform: Transform, claim_button: Vec2) -> anyhow::Result<HeadlessDevice> {
        let mut adb_shell_input = Command::new("adb")
            .arg("shell")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn adb shell for input")?;
        let mut adb_shell_screencap = Command::new("adb")
            .arg("shell")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn adb shell for screencap")?;

        Ok(HeadlessDevice {
            transform,
            claim_button,
            rgbas: Vec::new(),
            adb_shell_input_stdin: adb_shell_input
                .stdin
                .take()
                .context("take adb shell stdin for input")?,
            adb_shell_screencap_stdin: adb_shell_screencap
                .stdin
                .take()
                .context("take adb shell stdin for screencap")?,
            adb_shell_screencap_stdout: adb_shell_screencap
                .stdout
                .take()
                .context("take adb shell stdout for screencap")?,
        })
    }
}
