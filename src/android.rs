use std::io::Write;
use std::{
    io,
    process::{Child, Command, Stdio},
};

use serde::Deserialize;
use thiserror::Error;

use crate::puzzle::BoardPoke;

#[derive(Debug, Deserialize)]
struct TapPosition {
    x: i32,
    y: i32,
}

#[derive(Debug, Deserialize)]
pub struct TapperConfig {
    top_left_arrow: TapPosition,
    claim_button: TapPosition,
    arrow_diameter: i32,
}

impl TapperConfig {
    fn arrow_position(&self, &BoardPoke(x, y): &BoardPoke) -> TapPosition {
        let x: u8 = x.into();
        let y: u8 = y.into();
        let x: i32 = x.into();
        let y: i32 = y.into();
        TapPosition {
            x: self.arrow_diameter * x + self.top_left_arrow.x,
            y: self.arrow_diameter * y + self.top_left_arrow.y,
        }
    }
}

#[derive(Debug, Error)]
pub enum TapperTapError {
    #[error("stdin of adb shell is missing")]
    NoStdin,

    #[error("failed to write into stdin: {0}")]
    WriteIntoStdin(#[source] io::Error),
}

pub struct Tapper {
    config: TapperConfig,
    shell: Child,
}

impl Tapper {
    pub fn new(config: TapperConfig) -> io::Result<Tapper> {
        let shell = Command::new("adb")
            .arg("shell")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Tapper { config, shell })
    }

    fn write_into_stdin(&mut self, s: &[u8]) -> Result<(), TapperTapError> {
        self.shell
            .stdin
            .as_mut()
            .ok_or(TapperTapError::NoStdin)?
            .write_all(s)
            .map_err(TapperTapError::WriteIntoStdin)
    }

    pub fn tap_many(&mut self, ps: &[BoardPoke]) -> Result<(), TapperTapError> {
        let mut script: String = ps
            .iter()
            .map(|p| {
                let TapPosition { x, y } = self.config.arrow_position(p);
                format!("input tap {} {} &\n", x, y)
            })
            .collect();
        script += "wait\n";
        self.write_into_stdin(script.as_bytes())?;
        Ok(())
    }

    pub fn tap_claim_button(&mut self) -> Result<(), TapperTapError> {
        let TapPosition { x, y } = self.config.claim_button;
        self.write_into_stdin(format!("input tap {} {}\n", x, y).as_bytes())
    }
}
