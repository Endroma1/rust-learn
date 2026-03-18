use std::sync::mpsc::Receiver;

use crate::tic_tac_toe::{
    board::Board,
    common::{GameResult, Message, Player},
};

pub struct Game {
    turn: Player,
    rounds: u8,

    receiver: Receiver<Message>,

    board: Board,
}
impl Game {
    pub fn new(receiver: Receiver<Message>) -> Self {
        Self {
            turn: Player::X,
            rounds: 0,
            receiver,
            board: Board::default(),
        }
    }
    pub fn run(&mut self) {
        self.game_loop();
    }
    fn game_loop(&mut self) {
        while let Ok(m) = self.receiver.recv() {
            match m {
                Message::CurrentTurn(s) => {
                    if s.send(self.turn).is_err() {
                        break;
                    }
                }
                Message::Place { index, result } => {
                    let res = self.board.insert(self.turn, index);
                    let res = match res {
                        Ok(_) => {
                            self.handle_successful_placement();
                            Ok(self.check_round())
                        }
                        Err(e) => Err(e),
                    };

                    if result.send(res).is_err() {
                        break;
                    };
                }
                Message::Board(s) => {
                    if s.send(self.board).is_err() {
                        break;
                    }
                }
                Message::Quit => break,
            }
        }
    }
    fn handle_successful_placement(&mut self) {
        self.turn = match self.turn {
            Player::O => Player::X,
            Player::X => Player::O,
        };
        self.rounds += 1;
    }

    fn check_round(&self) -> Option<GameResult> {
        if self.board.has_win() {
            Some(GameResult::Win(self.turn))
        } else if self.rounds > 8 {
            Some(GameResult::Draw)
        } else {
            None
        }
    }
}
