use std::{
    sync::mpsc::{Sender, channel},
    thread::{self, JoinHandle},
};

use tokio::task::spawn_blocking;

use crate::tic_tac_toe::{common::Message, game::Game};

pub mod board;
pub mod common;
pub mod game;

/// Spawns a new game in a thread. Returns JoinHandle to the thread and a Sender to interact with
/// the game.
pub fn spawn_game() -> (JoinHandle<()>, Sender<Message>) {
    let (tx, rx) = channel();
    let handle = thread::spawn(move || {
        let mut game = Game::new(rx);
        game.run();
    });
    (handle, tx)
}

pub fn spawn_game_tokio() -> (tokio::task::JoinHandle<()>, Sender<Message>) {
    let (tx, rx) = channel();
    let h = spawn_blocking(|| {
        let mut game = Game::new(rx);
        game.run();
    });
    (h, tx)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use crate::tic_tac_toe::{
        board::{Board, InsertError},
        common::Message,
        spawn_game,
    };

    #[test]
    fn test_place() -> Result<(), InsertError> {
        let (handle, tx_game) = spawn_game();

        let (tx, rx) = channel();
        tx_game
            .send(super::common::Message::Place {
                index: 0,
                result: tx,
            })
            .expect("Game closed before expected");

        rx.recv().expect("Game closed before expected")?;

        let (tx, rx) = channel();
        tx_game
            .send(super::common::Message::Board(tx))
            .expect("Game closed before expected");

        let board = rx.recv().expect("Game closed before expected");
        let mut expected_board = Board::default();
        expected_board.insert(super::common::Player::X, 0)?;

        assert_eq!(board, expected_board);

        tx_game
            .send(Message::Quit)
            .expect("Game closed before expected");

        handle.join().unwrap();
        Ok(())
    }
    #[test]
    fn test_place_out_of_bounds() {
        let (handle, tx_game) = spawn_game();

        let (tx, rx) = channel();
        tx_game
            .send(super::common::Message::Place {
                index: 14,
                result: tx,
            })
            .expect("Game closed before expected");

        let res = rx.recv().expect("Game closed before expected");
        assert!(res.is_err());

        let (tx, rx) = channel();
        tx_game
            .send(super::common::Message::Board(tx))
            .expect("Game closed before expected");

        let board = rx.recv().expect("Game closed before expected");
        let expected_board = Board::default();

        assert_eq!(board, expected_board);

        tx_game
            .send(Message::Quit)
            .expect("Game closed before expected");

        handle.join().unwrap();
    }
}
