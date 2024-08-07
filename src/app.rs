use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{bail, Context};

use crate::{expert::Board, hex::Hex};

pub trait Device {
    fn wait_duration() -> Duration;
    fn detect_board(&mut self) -> anyhow::Result<Option<Board>>;
    fn tap_board(&mut self, taps: Hex<usize>) -> anyhow::Result<()>;
    fn tap_claim_button(&mut self) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoardState {
    Solved,
    Unsolved(Board),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerState {
    WaitForBoard,
    WaitForSolvedBoard,
    WaitForUnsolvedBoard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    Wait,
    Solve(Board),
    ClaimRewards,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerTransitionContext {
    now: Instant,
    board: Option<Board>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Player {
    state: PlayerState,
    last_transition: Instant,
}

impl Player {
    fn new(last_transition: Instant) -> Player {
        Player {
            state: PlayerState::WaitForBoard,
            last_transition,
        }
    }

    fn set_current_state(&mut self, now: Instant, new_state: PlayerState) {
        self.state = new_state;
        self.last_transition = now;
    }

    fn transition(&mut self, ctx: PlayerTransitionContext) -> anyhow::Result<Action> {
        let PlayerTransitionContext { now, board } = ctx;
        let elapsed = now
            .checked_duration_since(self.last_transition)
            .context("`ctx.now` is earlier than `self.last_transition`")?;
        let maybe_board_state = board.map(|b| {
            if b.is_solved() {
                BoardState::Solved
            } else {
                BoardState::Unsolved(b)
            }
        });

        let action = match (self.state, maybe_board_state) {
            (PlayerState::WaitForBoard, None) => {
                if elapsed > Duration::from_secs(10) {
                    bail!("waited for a board for {:?}", elapsed);
                }
                Action::Wait
            }
            (PlayerState::WaitForBoard, Some(BoardState::Unsolved(b))) => {
                self.set_current_state(now, PlayerState::WaitForSolvedBoard);
                Action::Solve(b)
            }
            (PlayerState::WaitForBoard, Some(BoardState::Solved)) => {
                self.set_current_state(now, PlayerState::WaitForUnsolvedBoard);
                Action::ClaimRewards
            }

            (PlayerState::WaitForSolvedBoard | PlayerState::WaitForUnsolvedBoard, None) => {
                self.set_current_state(now, PlayerState::WaitForBoard);
                Action::Wait
            }
            (PlayerState::WaitForSolvedBoard, Some(BoardState::Solved)) => {
                self.set_current_state(now, PlayerState::WaitForUnsolvedBoard);
                Action::ClaimRewards
            }
            (PlayerState::WaitForUnsolvedBoard, Some(BoardState::Unsolved(b))) => {
                self.set_current_state(now, PlayerState::WaitForSolvedBoard);
                Action::Solve(b)
            }

            // After solving the board until the screen updates. If the board
            // doesn't align, it's probably because some clicks didn't register.
            // Try solving the board again.
            (PlayerState::WaitForSolvedBoard, Some(BoardState::Unsolved(b))) => {
                if elapsed > Duration::from_secs(1) {
                    self.set_current_state(now, PlayerState::WaitForSolvedBoard);
                    Action::Solve(b)
                } else {
                    Action::Wait
                }
            }

            // After hitting the claim button until the screen updates
            (PlayerState::WaitForUnsolvedBoard, Some(BoardState::Solved)) => {
                if elapsed > Duration::from_secs(1) {
                    bail!("waited for unsolved board for {:?}", elapsed);
                } else {
                    Action::Wait
                }
            }
        };
        Ok(action)
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
                board,
            })
            .context("player transition")?;
        match action {
            Action::Wait => {
                sleep(D::wait_duration());
            }
            Action::Solve(b) => {
                device.tap_board(b.solve()).context("tap board")?;
            }
            Action::ClaimRewards => {
                device.tap_claim_button().context("tap claim button")?;
            }
        }
    }
}
