use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context};

use crate::{expert::Board, hex::Hex};

pub trait Device {
    fn detect_board(&mut self) -> anyhow::Result<Board>;
    fn tap_board(&mut self, taps: Hex<u8>) -> anyhow::Result<()>;
    fn tap_claim_button(&mut self) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnscreenBoard {
    Solved,
    Unsolved,
}

impl OnscreenBoard {
    fn new(board: &Board) -> Self {
        if board.is_solved() {
            Self::Solved
        } else {
            Self::Unsolved
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerState {
    Start,
    WaitForSolvedOnscreenBoard,
    WaitForUnsolvedOnscreenBoard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Wait,
    Solve,
    ClaimRewards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerTransitionContext {
    now: Instant,
    onscreen_board: OnscreenBoard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Player {
    state: PlayerState,
    last_transition: Instant,
}

impl Player {
    fn new(last_transition: Instant) -> Player {
        Player {
            state: PlayerState::Start,
            last_transition,
        }
    }

    fn set_current_state(&mut self, ctx: PlayerTransitionContext, new: PlayerState) {
        self.state = new;
        self.last_transition = ctx.now;
    }

    fn transition(&mut self, ctx: PlayerTransitionContext) -> anyhow::Result<Action> {
        let elapsed = match ctx.now.checked_duration_since(self.last_transition) {
            Some(x) => x,
            None => bail!("`ctx.now` is earlier than `self.last_transition`"),
        };

        match (self.state, ctx.onscreen_board) {
            (
                PlayerState::Start | PlayerState::WaitForSolvedOnscreenBoard,
                OnscreenBoard::Solved,
            ) => {
                self.set_current_state(ctx, PlayerState::WaitForUnsolvedOnscreenBoard);
                Ok(Action::ClaimRewards)
            }

            (
                PlayerState::Start | PlayerState::WaitForUnsolvedOnscreenBoard,
                OnscreenBoard::Unsolved,
            ) => {
                self.set_current_state(ctx, PlayerState::WaitForSolvedOnscreenBoard);
                Ok(Action::Solve)
            }

            // After solving the board until the screen updates. If the board
            // doesn't align, it's probably because some clicks didn't register.
            // Try solving the board again.
            (PlayerState::WaitForSolvedOnscreenBoard, OnscreenBoard::Unsolved) => {
                if elapsed > Duration::from_secs(2) {
                    self.set_current_state(ctx, PlayerState::WaitForSolvedOnscreenBoard);
                    Ok(Action::Solve)
                } else {
                    Ok(Action::Wait)
                }
            }

            // After hitting the claim button until the screen updates
            (PlayerState::WaitForUnsolvedOnscreenBoard, OnscreenBoard::Solved) => {
                if elapsed > Duration::from_secs(2) {
                    Err(anyhow!("waited for unsolved board for {:?}", elapsed))
                } else {
                    Ok(Action::Wait)
                }
            }
        }
    }
}

pub fn play<D>(mut device: D) -> anyhow::Result<()>
where
    D: Device,
{
    let mut player = Player::new(Instant::now());

    loop {
        let board = device.detect_board().context("detect board")?;
        let action = player
            .transition(PlayerTransitionContext {
                now: Instant::now(),
                onscreen_board: OnscreenBoard::new(&board),
            })
            .context("player transition")?;
        match action {
            Action::Wait => {
                // FIXME:
                sleep(Duration::from_millis(1));
            }
            Action::Solve => {
                // FIXME:
                let mut taps = Hex::from_fn(|_, _| 0u8);
                for (x, y) in board.solve() {
                    let x = x as usize - 1;
                    let y = y as usize - 1;
                    *taps.at_mut(x, y).unwrap() += 1;
                }
                for (_, _, n) in taps.enumerate_mut() {
                    *n %= 6;
                }
                device.tap_board(taps).context("tap board")?;
            }
            Action::ClaimRewards => {
                device.tap_claim_button().context("tap claim button")?;
            }
        }
    }
}
