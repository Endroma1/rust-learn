use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::tic_tac_toe::board::{Board, InsertError};

pub enum Message {
    /// Place piece based on current turn. Returns error if invalid move, returns game result if a
    /// winner or draw is reached.
    Place {
        index: usize,
        result: oneshot::Sender<Result<Option<GameResult>, InsertError>>,
    },
    CurrentTurn(oneshot::Sender<Player>),
    /// Get the board for rendering.
    Board(oneshot::Sender<Board>),
    Quit,
}
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Player {
    X,
    O,
}
impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::X => write!(f, "x"),
            Player::O => write!(f, "o"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameResult {
    Win(Player),
    Draw,
}
