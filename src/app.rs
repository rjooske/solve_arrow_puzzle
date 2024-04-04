use std::{
    iter::once,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context};
use itertools::Itertools;

use crate::expert::{Arrow, Board};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    pub const BLACK: Pixel = Pixel { r: 0, g: 0, b: 0 };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screenshot {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>,
}

impl Screenshot {
    fn at(&self, x: usize, y: usize) -> Option<Pixel> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(self.pixels[x + self.width * y])
        }
    }
}

pub trait Screen {
    fn shoot(&mut self) -> Screenshot;
    fn tap_many<I>(&mut self, taps: I)
    where
        I: Iterator<Item = (i32, i32)>;

    fn tap(&mut self, x: i32, y: i32) {
        self.tap_many(once((x, y)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
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
struct Transform {
    top_arrow: Vec2,
    arrow_diameter: f64,
    axis_a: Vec2,
    axis_b: Vec2,
}

impl Transform {
    fn new(top_arrow: Vec2, bottom_arrow: Vec2) -> Transform {
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

    fn index_to_click_position(&self, x: u8, y: u8) -> Vec2 {
        let offset = Vec2 { x: 0.0, y: 1.0 }.scale(0.5 * self.arrow_diameter);
        self.index_to_position(x, y).add(offset)
    }

    fn positions(&self) -> impl Iterator<Item = Vec2> + '_ {
        Board::POSITIONS
            .iter()
            .map(|&(x, y)| self.index_to_position(x, y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnscreenArrow {
    Aligned,
    Unaligned(Arrow),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OnscreenBoard {
    Aligned,
    Unaligned(Board),
}

fn closest_onscreen_arrow(
    red_to_onscreen_arrow: &[(u8, OnscreenArrow)],
    target_red: f64,
) -> anyhow::Result<(f64, OnscreenArrow)> {
    red_to_onscreen_arrow
        .iter()
        .map(|&(red, arrow)| ((red as f64 - target_red).abs(), arrow))
        .min_by(|(a, _), (b, _)| a.total_cmp(b))
        .context("empty red -> onscreen arrow mapping")
}

fn find_onscreen_board(
    red_to_onscreen_arrow: &[(u8, OnscreenArrow)],
    arrow_reds: &[f64],
    diff_threshold: f64,
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
        bail!(
            "pair `{:?}` has diff above threshold `{}`: {:?}",
            pair,
            diff_threshold,
            diff_arrow_pairs
        );
    }

    let unaligned_arrows = diff_arrow_pairs
        .iter()
        .filter_map(|&(_, a)| match a {
            OnscreenArrow::Aligned => None,
            OnscreenArrow::Unaligned(a) => Some(a),
        })
        .collect::<Vec<_>>();

    if !(unaligned_arrows.is_empty() || unaligned_arrows.len() == diff_arrow_pairs.len()) {
        bail!(
            "board is neither aligned nor unaligned: {:?}",
            diff_arrow_pairs
        );
    }

    if unaligned_arrows.is_empty() {
        Ok(OnscreenBoard::Aligned)
    } else {
        Ok(OnscreenBoard::Unaligned(Board::from_arrows(
            unaligned_arrows.iter().copied(),
        )))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerState {
    Start,
    WaitForAlignedOnscreenBoard,
    WaitForUnalignedOnscreenBoard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action<'a> {
    Nothing,
    Solve(&'a Board),
    ClaimRewards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerTransitionContext<'a> {
    now: Instant,
    onscreen_board: &'a OnscreenBoard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Player {
    current: PlayerState,
    last_transition: Instant,
}

impl Player {
    fn new(last_transition: Instant) -> Player {
        Player {
            current: PlayerState::Start,
            last_transition,
        }
    }

    fn set_current_state(&mut self, ctx: PlayerTransitionContext, new: PlayerState) {
        self.current = new;
        self.last_transition = ctx.now;
    }

    fn transition<'a>(
        &'a mut self,
        ctx: PlayerTransitionContext<'a>,
    ) -> anyhow::Result<Action<'a>> {
        let elapsed = match ctx.now.checked_duration_since(self.last_transition) {
            Some(x) => x,
            None => bail!("`ctx.now` is earlier than `self.last_transition`"),
        };

        match (self.current, ctx.onscreen_board) {
            (
                PlayerState::Start | PlayerState::WaitForAlignedOnscreenBoard,
                OnscreenBoard::Aligned,
            ) => {
                self.set_current_state(ctx, PlayerState::WaitForUnalignedOnscreenBoard);
                Ok(Action::ClaimRewards)
            }

            (
                PlayerState::Start | PlayerState::WaitForUnalignedOnscreenBoard,
                OnscreenBoard::Unaligned(b),
            ) => {
                self.set_current_state(ctx, PlayerState::WaitForAlignedOnscreenBoard);
                Ok(Action::Solve(b))
            }

            // After solving the board until the screen updates. If the board
            // doesn't align, it's probably because some clicks didn't register.
            // Try solving the board again.
            (PlayerState::WaitForAlignedOnscreenBoard, OnscreenBoard::Unaligned(b)) => {
                if elapsed > Duration::from_secs(1) {
                    self.set_current_state(ctx, PlayerState::WaitForAlignedOnscreenBoard);
                    Ok(Action::Solve(b))
                } else {
                    Ok(Action::Nothing)
                }
            }

            // After hitting the claim button until the screen updates
            (PlayerState::WaitForUnalignedOnscreenBoard, OnscreenBoard::Aligned) => {
                if elapsed > Duration::from_secs(5) {
                    Err(anyhow!("waited for unaligned board for {:?}", elapsed))
                } else {
                    Ok(Action::Nothing)
                }
            }
        }
    }
}

pub fn play<S>(mut screen: S) -> anyhow::Result<()>
where
    S: Screen,
{
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
    let top = Vec2 { x: 236.0, y: 357.0 };
    let bottom = Vec2 { x: 236.0, y: 767.0 };
    let claim = Vec2 { x: 236.0, y: 904.0 };
    let red_to_onscreen_arrow: Vec<(u8, OnscreenArrow)> = vec![
        (25, OnscreenArrow::Aligned),
        (16, OnscreenArrow::Unaligned(Arrow(0))),
        (28, OnscreenArrow::Unaligned(Arrow(1))),
        (42, OnscreenArrow::Unaligned(Arrow(2))),
        (55, OnscreenArrow::Unaligned(Arrow(3))),
        (69, OnscreenArrow::Unaligned(Arrow(4))),
        (83, OnscreenArrow::Unaligned(Arrow(5))),
    ];
    // background red: 49

    let transform = Transform::new(top, bottom);
    let mut player = Player::new(Instant::now());

    loop {
        let screenshot = screen.shoot();
        let arrow_reds = transform
            .positions()
            .map(|p| {
                let (x, y) = p.round_as_i32();
                // FIXME: unwrap
                (-2..=2)
                    .flat_map(|dy| (-2..=2).map(move |dx| (dx, dy)))
                    .map(|(dx, dy)| {
                        let x = x + dx;
                        let y = y + dy;
                        screenshot.at(x as usize, y as usize).unwrap().r as f64
                    })
                    .sum::<f64>()
                    / 25.0
            })
            .collect_vec();

        // println!("{:?}", arrow_reds);
        // let mut ps = screenshot.pixels;
        // for p in transform.positions() {
        //     let (x, y) = p.round_as_usize();
        //     ps[x + screenshot.width * y] = Pixel { r: 255, g: 0, b: 0 };
        // }
        // let bytes = ps.iter().flat_map(|&p| [p.r, p.g, p.b]).collect_vec();
        // write("zzz", bytes).unwrap();

        let onscreen_board = find_onscreen_board(&red_to_onscreen_arrow, &arrow_reds, 2.0)
            .context("find onscreen board")?;

        let action = player
            .transition(PlayerTransitionContext {
                now: Instant::now(),
                onscreen_board: &onscreen_board,
            })
            .context("player transition")?;
        match action {
            Action::Nothing => sleep(Duration::from_millis(10)),
            Action::Solve(board) => {
                let taps = board
                    .clone()
                    .solve()
                    .into_iter()
                    .map(|(ix, iy)| transform.index_to_click_position(ix, iy).round_as_i32());
                screen.tap_many(taps);
            }
            Action::ClaimRewards => {
                let (x, y) = claim.round_as_i32();
                screen.tap(x, y);
            }
        }
    }
}
