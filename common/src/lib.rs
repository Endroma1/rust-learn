use std::{
    sync::mpsc::{Sender, channel},
    thread::{self, JoinHandle},
};

use tokio::{spawn, task::spawn_blocking};

use crate::tic_tac_toe::{common::Message, game::Game};

pub mod tic_tac_toe;
