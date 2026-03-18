use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::tic_tac_toe::common::Player;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    board: [Option<Player>; 9],
}
impl Board {
    /// Returns the winning player if winning condition is satisfied.
    pub fn has_win(&self) -> bool {
        // For visualization:
        // [0, 1, 2]
        // [3, 4, 5]
        // [6, 7, 8]

        // Rows
        for i in 0..3 {
            let b = i * 3;
            if check_line(
                self.board[b].as_ref(),
                self.board[b + 1].as_ref(),
                self.board[b + 2].as_ref(),
            ) {
                return true;
            };
        }

        // Columns
        for i in 0..3 {
            if check_line(
                self.board[i].as_ref(),
                self.board[i + 3].as_ref(),
                self.board[i + 6].as_ref(),
            ) {
                return true;
            };
        }

        // \ Diagonal
        if check_line(
            self.board[0].as_ref(),
            self.board[4].as_ref(),
            self.board[8].as_ref(),
        ) {
            return true;
        }

        // / Diagonal
        if check_line(
            self.board[2].as_ref(),
            self.board[4].as_ref(),
            self.board[6].as_ref(),
        ) {
            return true;
        }
        false
    }

    /// Validates the index and inserts player into a cell in the board.
    pub fn insert(&mut self, player: Player, index: usize) -> Result<&mut Self, InsertError> {
        let cell = self
            .board
            .get_mut(index)
            .ok_or(InsertError::IndexOutOfBounds(index))?;

        if cell.is_some() {
            return Err(InsertError::IndexOccupied(index));
        }

        *cell = Some(player);
        Ok(self)
    }
}
fn check_line(cell1: Option<&Player>, cell2: Option<&Player>, cell3: Option<&Player>) -> bool {
    cell1 != None && cell1 == cell2 && cell1 == cell3
}
#[derive(Debug, Serialize, Deserialize)]
pub enum InsertError {
    IndexOutOfBounds(usize),
    IndexOccupied(usize),
}
impl Display for InsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexOutOfBounds(i) => write!(f, "Index {} is out of bounds", i),
            Self::IndexOccupied(i) => write!(f, "Index {} is already occupied", i),
        }
    }
}
