use std::{
    env::temp_dir,
    fmt::Write as _,
    fs::File,
    io::{Read as _, Write},
    process::{ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use itertools::Itertools;
use nix::{sys::stat::Mode, unistd::mkfifo};
use rand::random;

use crate::app::{Pixel, Screen, Screenshot};

#[derive(Debug)]
/// FIXME: currently doesn't clean up anything
pub struct HeadlessScreen {
    width: usize,
    height: usize,
    pixels: Arc<Mutex<Vec<Pixel>>>,
    adb_shell_stdin: ChildStdin,
}

impl Screen for HeadlessScreen {
    fn shoot(&mut self) -> Screenshot {
        Screenshot {
            width: self.width,
            height: self.height,
            pixels: self.pixels.lock().unwrap().clone(),
        }
    }

    fn tap_many<I>(&mut self, taps: I)
    where
        I: Iterator<Item = (i32, i32)>,
    {
        let mut commands = String::new();
        for (x, y) in taps {
            let x = (x as f64 / 1024.0 * 3140.0).round() as i32;
            let y = (y as f64 / 1024.0 * 3140.0).round() as i32;
            writeln!(commands, "input tap {} {} &", x, y).unwrap();
        }
        writeln!(commands, "wait").unwrap();
        self.adb_shell_stdin.write_all(commands.as_bytes()).unwrap();
        self.adb_shell_stdin.flush().unwrap();
    }
}

impl HeadlessScreen {
    pub fn new(width: usize, height: usize) -> HeadlessScreen {
        let mut fifo_path = temp_dir();
        fifo_path.push(format!("video_{}", random::<u64>()));
        mkfifo(&fifo_path, Mode::S_IRUSR | Mode::S_IWUSR).unwrap();

        let _scrcpy = Command::new("scrcpy")
            .args(["--no-clipboard-autosync", "--no-audio", "--no-playback"])
            .args(["--max-size", "1024"])
            .args(["--video-bit-rate", "16M"])
            .args(["--record-format", "mkv"])
            .args(["--record", &fifo_path.to_string_lossy()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut ffmpeg = Command::new("ffmpeg")
            .args(["-r", "999"])
            .args(["-f", "matroska"])
            .args(["-i", "-"])
            .args(["-pix_fmt", "rgb24"])
            .args(["-f", "rawvideo"])
            .args(["-"])
            .stdin(File::open(&fifo_path).unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let pixels = Arc::new(Mutex::new(vec![Pixel::BLACK; width * height]));

        {
            let pixels = pixels.clone();
            thread::spawn(move || {
                let mut ffmpeg_stdout = ffmpeg.stdout.take().unwrap();
                let mut rgbs = vec![0u8; 3 * width * height];
                loop {
                    ffmpeg_stdout.read_exact(&mut rgbs).unwrap();
                    let mut pixels = pixels.lock().unwrap();
                    pixels.clear();
                    pixels.extend(rgbs.iter().tuples().map(|(&r, &g, &b)| Pixel { r, g, b }));
                }
            });
        }

        let mut adb_shell = Command::new("adb")
            .arg("shell")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let adb_shell_stdin = adb_shell.stdin.take().unwrap();

        HeadlessScreen {
            width,
            height,
            pixels,
            adb_shell_stdin,
        }
    }
}
