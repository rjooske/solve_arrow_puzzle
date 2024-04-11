use std::{
    fmt::{Debug, Write as _},
    io::{self, Read, Write},
    iter::{once, repeat},
    net::{SocketAddr, TcpListener, TcpStream},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use anyhow::{anyhow, bail, Context};
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, MouseButton, MouseControllable};
use flate2::read::GzDecoder;
use itertools::Itertools;
use phf::phf_map;
use rand::random;
use scrap::{Capturer, Display};

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

    fn round_as_u32(self) -> (u32, u32) {
        (self.x.round() as u32, self.y.round() as u32)
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
pub struct HeadlessDevice {
    width: usize,
    claim_button: Vec2,
    arrow_positions: Hex<(usize, usize)>,
    screencap_output: Vec<u8>,
    adb_shell_input: Child,
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

impl Drop for HeadlessDevice {
    fn drop(&mut self) {
        let _ = self.adb_shell_input.kill();
        let _ = self.adb_shell_input.wait();
    }
}

impl Device for HeadlessDevice {
    fn detect_board(&mut self) -> anyhow::Result<Board> {
        let output = Command::new("adb")
            .arg("shell")
            .arg("screencap | gzip -c -k -1 /dev/stdin")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn screencap and gzip")?
            .wait_with_output()
            .context("wait for screencap and gzip")?;
        self.screencap_output.clear();
        GzDecoder::new(output.stdout.as_slice())
            .read_to_end(&mut self.screencap_output)
            .context("decode gzipped screencap output")?;

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
            adb_shell_input,
        })
    }
}

struct ScreenView<'a> {
    bgras: &'a [u8],
    width: usize,
}

struct Screen(Capturer);

impl Debug for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Screen").field(&"-").finish()
    }
}

impl Screen {
    fn new(capturer: Capturer) -> Screen {
        Screen(capturer)
    }

    fn view_and_map<F, T>(&mut self, f: F) -> io::Result<T>
    where
        F: FnOnce(ScreenView) -> T,
    {
        let Screen(capturer) = self;
        let width = capturer.width();

        loop {
            match capturer.frame() {
                Ok(frame) => {
                    return Ok(f(ScreenView {
                        bgras: &frame,
                        width,
                    }));
                }
                Err(err) => {
                    if err.kind() == io::ErrorKind::WouldBlock {
                        sleep(Duration::from_millis(1));
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct OnscreenDevice {
    arrow_positions: Hex<(i32, i32)>,
    claim_button_x: i32,
    claim_button_y: i32,
    device_state: DeviceState,
    screen: Screen,
    enigo: Enigo,
    scrcpy: Child,
}

impl Drop for OnscreenDevice {
    fn drop(&mut self) {
        let _ = self.scrcpy.kill();
        let _ = self.scrcpy.wait();
    }
}

impl Device for OnscreenDevice {
    fn detect_board(&mut self) -> anyhow::Result<Board> {
        if self.device_state.get_keys().contains(&Keycode::Backspace) {
            bail!("interrupted");
        }
        if let Some(status) = self.scrcpy.try_wait().context("try waiting for scrcpy")? {
            bail!("scrcpy exited: {}", status);
        }

        self.screen
            .view_and_map(|view| {
                let arrows: Vec<_> = self
                    .arrow_positions
                    .enumerate()
                    .map(|(_, _, &(x, y))| {
                        let x = x as usize;
                        let y = y as usize;
                        let r = view.bgras[4 * (x + view.width * y) + 2];
                        RED_TO_ARROW
                            .get(&r)
                            .copied()
                            .with_context(|| format!("no arrows correspond to red value {}", r))
                    })
                    .try_collect()?;
                anyhow::Ok(Board::from_arrows(arrows))
            })
            .context("view screen")?
            .context("map screen view")
    }

    fn tap_board(&mut self, taps: Hex<u8>) -> anyhow::Result<()> {
        let taps = taps
            .enumerate()
            .zip(self.arrow_positions.enumerate())
            .filter_map(
                |((_, _, &n), (_, _, &(x, y)))| {
                    if n == 0 {
                        None
                    } else {
                        Some((x, y, n))
                    }
                },
            );
        for (x, y, n) in taps {
            self.enigo.mouse_move_to(x + 20, y);
            sleep(Duration::from_micros(1000));
            for _ in 0..n {
                self.enigo.mouse_down(MouseButton::Left);
                self.enigo.mouse_up(MouseButton::Left);
            }
            sleep(Duration::from_micros(1500));
        }
        Ok(())
    }

    fn tap_claim_button(&mut self) -> anyhow::Result<()> {
        self.enigo
            .mouse_move_to(self.claim_button_x, self.claim_button_y);
        sleep(Duration::from_micros(1000));
        self.enigo.mouse_down(MouseButton::Left);
        sleep(Duration::from_micros(1000));
        self.enigo.mouse_up(MouseButton::Left);
        sleep(Duration::from_micros(1000));
        Ok(())
    }
}

impl OnscreenDevice {
    pub fn new(transform: Transform, claim_button: Vec2) -> anyhow::Result<OnscreenDevice> {
        let arrow_positions = Hex::from_fn(|x, y| transform.index_to_position(x, y).round_as_i32());
        let (claim_button_x, claim_button_y) = claim_button.round_as_i32();
        let screen = Screen::new(
            Capturer::new(Display::primary().context("get primary display")?)
                .context("create capturer")?,
        );
        let scrcpy = std::process::Command::new("scrcpy")
            .arg("--window-x=0")
            .arg("--window-y=0")
            .arg("--video-bit-rate=64M")
            .arg("--no-audio")
            .arg("--no-clipboard-autosync")
            .arg("--always-on-top")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn scrcpy")?;

        let device_state = DeviceState::new();
        while !device_state.get_keys().contains(&Keycode::Backspace) {
            sleep(Duration::from_millis(10));
        }
        while device_state.get_keys().contains(&Keycode::Backspace) {
            sleep(Duration::from_millis(10));
        }

        Ok(OnscreenDevice {
            arrow_positions,
            claim_button_x,
            claim_button_y,
            device_state: DeviceState::new(),
            screen,
            enigo: Enigo::new(),
            scrcpy,
        })
    }
}

#[derive(Debug)]
pub struct ScrcpyServerDevice {
    screen_width: usize,
    screen_height: usize,
    claim_button_x: u32,
    claim_button_y: u32,
    arrow_tap_positions: Hex<(u32, u32)>,
    luma_sample_positions: Hex<Vec<(usize, usize)>>,
    video_server: Child,
    control_server: Child,
    ffmpeg: Child,
    control_stream: TcpStream,
    lumas: Arc<Mutex<Vec<u8>>>,
}

impl Drop for ScrcpyServerDevice {
    fn drop(&mut self) {
        let _ = self.video_server.kill();
        let _ = self.video_server.wait();
        let _ = self.control_server.kill();
        let _ = self.control_server.wait();
        let _ = self.ffmpeg.kill();
        let _ = self.ffmpeg.wait();
    }
}

impl Device for ScrcpyServerDevice {
    fn detect_board(&mut self) -> anyhow::Result<Board> {
        static LUMA_TO_ARROW: phf::Map<u8, Arrow> = phf_map! {
            39u8 => Arrow(0),
            31u8 => Arrow(0),
            42u8 => Arrow(1),
            54u8 => Arrow(2),
            65u8 => Arrow(3),
            77u8 => Arrow(4),
            89u8 => Arrow(5),
        };

        let lumas = self
            .lumas
            .lock()
            .map_err(|err| anyhow!("failed to take the lock for lumas: {}", err))?;
        let arrows: Vec<_> = self
            .luma_sample_positions
            .enumerate()
            .map(|(_, _, ps)| {
                let luma = ps
                    .iter()
                    .map(|&(x, y)| lumas[x + self.screen_width * y] as f64)
                    .sum::<f64>()
                    / Self::SAMPLE_COUNT_PER_ARROW as f64;
                let luma = luma.round() as u8;
                LUMA_TO_ARROW
                    .get(&luma)
                    .copied()
                    .with_context(|| format!("no arrows correspond to luma value {}", luma))
            })
            .try_collect()?;
        Ok(Board::from_arrows(arrows))
    }

    fn tap_board(&mut self, taps: Hex<u8>) -> anyhow::Result<()> {
        let taps = taps
            .enumerate()
            .zip(self.arrow_tap_positions.enumerate())
            .flat_map(|((_, _, &n), (_, _, &(x, y)))| repeat((x, y)).take(n.into()));
        let taps = Self::serialize_taps(self.screen_width, self.screen_height, taps);
        self.control_stream.write_all(&taps).context("tap board")
    }

    fn tap_claim_button(&mut self) -> anyhow::Result<()> {
        let taps = Self::serialize_taps(
            self.screen_width,
            self.screen_height,
            once((self.claim_button_x, self.claim_button_y)),
        );
        self.control_stream
            .write_all(&taps)
            .context("tap claim button")
    }
}

impl ScrcpyServerDevice {
    const SAMPLE_COUNT_PER_ARROW: usize = 8;

    pub fn new(
        screen_width: usize,
        screen_height: usize,
        claim_button: Vec2,
        transform: Transform,
        scrcpy_server_path: &str,
        scrcpy_video_port: u16,
        scrcpy_control_port: u16,
    ) -> anyhow::Result<ScrcpyServerDevice> {
        use std::f64::consts::PI;

        let (claim_button_x, claim_button_y) = claim_button.round_as_u32();
        let arrow_tap_positions =
            Hex::from_fn(|x, y| transform.index_to_position(x, y).round_as_u32());
        let luma_sample_positions = Hex::from_fn(|x, y| {
            let center = transform.index_to_position(x, y);
            let diff = Vec2::new(1.0, 0.0).scale(transform.arrow_diameter / 2.0 * 0.75);
            (0..Self::SAMPLE_COUNT_PER_ARROW)
                .map(|i| {
                    let angle = 2.0 * PI * i as f64 / Self::SAMPLE_COUNT_PER_ARROW as f64;
                    center.add(diff.rotate(angle)).round_as_usize()
                })
                .collect_vec()
        });

        let adb_push_status = Command::new("adb")
            .args([
                "push",
                scrcpy_server_path,
                "/data/local/tmp/scrcpy-server-manual.jar",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut c| c.wait())
            .context("push scrcpy server to android device")?;
        if !adb_push_status.success() {
            bail!("adb push failed: {}", adb_push_status);
        }

        let video_scid = random::<u32>() & 0x7fffffff;
        let control_scid = random::<u32>() & 0x7fffffff;
        let adb_reverse_video_status = Command::new("adb")
            .args([
                "reverse",
                &format!("localabstract:scrcpy_{:08x}", video_scid),
                &format!("tcp:{}", scrcpy_video_port),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut c| c.wait())
            .context("create reverse tcp tunnel for video stream")?;
        let adb_reverse_control_status = Command::new("adb")
            .args([
                "reverse",
                &format!("localabstract:scrcpy_{:08x}", control_scid),
                &format!("tcp:{}", scrcpy_control_port),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut c| c.wait())
            .context("create reverse tcp tunnel for control stream")?;
        if !adb_reverse_video_status.success() {
            bail!(
                "adb reverse for video stream failed: {}",
                adb_reverse_video_status
            );
        }
        if !adb_reverse_control_status.success() {
            bail!(
                "adb reverse for control stream failed: {}",
                adb_reverse_control_status
            );
        }

        let video_tcp_listener =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], scrcpy_video_port)))
                .context("listen to tcp connection for video stream")?;
        let control_tcp_listener =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], scrcpy_control_port)))
                .context("listen to tcp connection for control stream")?;
        let video_server = Command::new("adb")
            .args([
                "shell",
                "CLASSPATH=/data/local/tmp/scrcpy-server-manual.jar",
                "app_process",
                "/",
                "com.genymobile.scrcpy.Server",
                "2.4",
                &format!("scid={:08x}", video_scid),
                "video=true",
                "audio=false",
                "control=false",
                "raw_stream=true",
                "max_fps=60",
                "video_bit_rate=67108864", // 64 * 1024 * 1024
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("start video server")?;
        let control_server = Command::new("adb")
            .args([
                "shell",
                "CLASSPATH=/data/local/tmp/scrcpy-server-manual.jar",
                "app_process",
                "/",
                "com.genymobile.scrcpy.Server",
                "2.4",
                &format!("scid={:08x}", control_scid),
                "video=false",
                "audio=false",
                "control=true",
                "raw_stream=true",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("start control server")?;
        let (mut video_stream, _) = video_tcp_listener
            .accept()
            .context("accept tcp connection for video stream")?;
        let (control_stream, _) = control_tcp_listener
            .accept()
            .context("accept tcp connection for control stream")?;

        let adb_reverse_remove_status = Command::new("adb")
            .args(["reverse", "--remove-all"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut c| c.wait())
            .context("remove all reverse tcp tunnels")?;
        if !adb_reverse_remove_status.success() {
            bail!(
                "failed to remove all reverse tcp tunnels: {}",
                adb_reverse_remove_status
            );
        }

        let mut ffmpeg = Command::new("ffmpeg")
            .args(["-re"])
            .args(["-flags", "low_delay"])
            .args(["-f", "h264"])
            .args(["-c:v", "h264"])
            .args(["-i", "-"])
            .args(["-pix_fmt", "yuv420p"])
            .args(["-f", "rawvideo"])
            .args(["-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("spawn ffmpeg")?;
        let mut ffmpeg_stdin = ffmpeg.stdin.take().context("take ffmpeg stdin")?;
        let mut ffmpeg_stdout = ffmpeg.stdout.take().context("take ffmpeg stdout")?;

        thread::spawn(move || {
            let mut buf = vec![0u8; 1 << 20];
            loop {
                let read_size = video_stream.read(&mut buf).unwrap();
                ffmpeg_stdin.write_all(&buf[0..read_size]).unwrap();
            }
        });

        let lumas_len = screen_width * screen_height;
        let lumas = Arc::new(Mutex::new(vec![0u8; lumas_len]));
        {
            let lumas = lumas.clone();
            thread::spawn(move || {
                let yuvs_len = 3 * screen_width * screen_height / 2;
                let mut yuvs = vec![0u8; yuvs_len];
                loop {
                    ffmpeg_stdout.read_exact(&mut yuvs).unwrap();
                    lumas.lock().unwrap().clone_from_slice(&yuvs[0..lumas_len]);
                }
            });
        }

        let mut device = ScrcpyServerDevice {
            screen_width,
            screen_height,
            claim_button_x,
            claim_button_y,
            arrow_tap_positions,
            luma_sample_positions,
            video_server,
            control_server,
            ffmpeg,
            control_stream,
            lumas,
        };
        while device.detect_board().is_err() {
            thread::sleep(Duration::from_millis(100));
        }

        Ok(device)
    }

    fn serialize_taps<I>(screen_width: usize, screen_height: usize, taps: I) -> Vec<u8>
    where
        I: IntoIterator<Item = (u32, u32)>,
    {
        // Test case: https://github.com/Genymobile/scrcpy/blob/206809a99affad9a7aa58fcf7593cea71f48954d/app/tests/test_control_msg_serialize.c#L77
        // Actual usage: https://github.com/Genymobile/scrcpy/blob/206809a99affad9a7aa58fcf7593cea71f48954d/app/src/input_manager.c#L363

        const SC_CONTROL_MSG_TYPE_INJECT_TOUCH_EVENT: u8 = 2;
        const AMOTION_EVENT_ACTION_DOWN: u8 = 0;
        const AMOTION_EVENT_ACTION_UP: u8 = 1;
        const DUMMY_POINTER_ID: [u8; 8] = u64::to_be_bytes(0x1234567887654321);

        let mut payload = vec![];
        for (x, y) in taps {
            payload.extend([
                SC_CONTROL_MSG_TYPE_INJECT_TOUCH_EVENT,
                AMOTION_EVENT_ACTION_DOWN,
            ]);
            payload.extend(DUMMY_POINTER_ID);
            payload.extend(u32::to_be_bytes(x));
            payload.extend(u32::to_be_bytes(y));
            payload.extend(u16::to_be_bytes(screen_width as u16));
            payload.extend(u16::to_be_bytes(screen_height as u16));
            payload.extend(u16::to_be_bytes(0xffff));
            payload.extend(u32::to_be_bytes(0));
            payload.extend(u32::to_be_bytes(0));

            payload.extend([
                SC_CONTROL_MSG_TYPE_INJECT_TOUCH_EVENT,
                AMOTION_EVENT_ACTION_UP,
            ]);
            payload.extend(DUMMY_POINTER_ID);
            payload.extend(u32::to_be_bytes(x));
            payload.extend(u32::to_be_bytes(y));
            payload.extend(u16::to_be_bytes(screen_width as u16));
            payload.extend(u16::to_be_bytes(screen_height as u16));
            payload.extend(u16::to_be_bytes(0));
            payload.extend(u32::to_be_bytes(0));
            payload.extend(u32::to_be_bytes(0));
        }
        payload
    }
}
