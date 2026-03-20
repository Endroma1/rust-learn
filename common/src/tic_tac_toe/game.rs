use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::tic_tac_toe::{
    board::{Board, InsertError},
    common::{GameResult, Message, Player},
};

/// Ref counted through Sender
#[derive(Debug, Clone)]
pub struct Game {
    handle: Sender<Message>,
}
impl Game {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(|| {
            let mut game = GameLoop::new(rx);
            game.run();
        });

        Self { handle: tx }
    }
    pub fn place(&self, index: usize) -> Result<Option<GameResult>, InsertError> {
        let (tx, rx) = oneshot::channel();
        self.handle
            .send(Message::Place {
                index: index,
                result: tx,
            })
            .unwrap();
        rx.recv().unwrap()
    }
    pub fn board(&self) -> Board {
        let (tx, rx) = oneshot::channel();
        self.handle.send(Message::Board(tx)).unwrap();
        rx.recv().unwrap()
    }
    pub fn current_turn(&self) -> Player {
        let (tx, rx) = oneshot::channel();
        self.handle.send(Message::CurrentTurn(tx)).unwrap();
        rx.recv().unwrap()
    }
    pub fn quit(&self) {
        self.handle.send(Message::Quit).unwrap();
    }
}

pub struct GameLoop {
    turn: Player,
    rounds: u8,
    should_stop: bool,

    receiver: Receiver<Message>,

    board: Board,
}
impl GameLoop {
    pub fn new(receiver: Receiver<Message>) -> Self {
        Self {
            turn: Player::X,
            rounds: 0,
            should_stop: true,
            receiver,
            board: Board::default(),
        }
    }
    pub fn run(&mut self) {
        self.game_loop_blocking();
    }
    fn game_loop_blocking(&mut self) {
        while !self.should_stop {
            let m = match self.receiver.recv() {
                Ok(m) => m,
                Err(_) => {
                    self.should_stop = true;
                    return;
                }
            };

            self.handle_message(m);
        }
    }
    fn handle_message(&mut self, m: Message) {
        match m {
            Message::CurrentTurn(s) => {
                if s.send(self.turn).is_err() {
                    self.should_stop = true;
                    return;
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
                    self.should_stop = true;
                    return;
                };
            }
            Message::Board(s) => {
                if s.send(self.board).is_err() {
                    self.should_stop = true;
                    return;
                }
            }
            Message::Quit => {
                self.should_stop = true;
                return;
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
