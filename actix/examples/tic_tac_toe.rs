use std::{
    fmt::Display,
    sync::mpsc::{Sender, channel},
};

use actix_web::{
    App, HttpResponse, HttpServer, Responder, Result,
    error::{self},
    get,
    http::StatusCode,
    post, web,
};
use common::tic_tac_toe::{board::InsertError, common::Message, spawn_game_tokio};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (handle, tx) = spawn_game_tokio();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State::new(tx.clone())))
            .service(turn)
            .service(board)
            .service(quit)
            .service(place)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    handle.await?;
    Ok(())
}

pub struct State {
    pub game_api: Sender<Message>,
}
impl State {
    pub fn new(api: Sender<Message>) -> Self {
        Self { game_api: api }
    }
}

#[get("/turn")]
async fn turn(state: web::Data<State>) -> Result<impl Responder> {
    let current_turn = spawn_blocking(move || -> Result<_, Error> {
        let (tx, rx) = channel();

        state
            .game_api
            .send(Message::CurrentTurn(tx))
            .map_err(|_| Error::GameQuit)?;
        let current_turn = rx.recv().unwrap();
        Ok(current_turn)
    })
    .await
    .unwrap()?;

    let resp = HttpResponse::Ok().body(current_turn.to_string());
    Ok(resp)
}

#[post("/quit")]
async fn quit(state: web::Data<State>) -> impl Responder {
    let _ = state.game_api.send(Message::Quit);
    HttpResponse::new(StatusCode::OK)
}

#[get("/board")]
async fn board(state: web::Data<State>) -> Result<impl Responder> {
    let board = spawn_blocking(move || -> Result<_, Error> {
        let (tx, rx) = channel();
        state
            .game_api
            .send(Message::Board(tx))
            .map_err(|_| Error::GameQuit)?;
        let board = rx.recv().unwrap();
        Ok(board)
    })
    .await
    .unwrap()?;

    Ok(web::Json(board))
}

#[post("/place")]
async fn place(
    req_body: web::Json<PlaceRequest>,
    state: web::Data<State>,
) -> Result<impl Responder> {
    let res = spawn_blocking(move || -> Result<_, Error> {
        let (tx, rx) = channel();
        state
            .game_api
            .send(Message::Place {
                index: req_body.index,
                result: tx,
            })
            .map_err(|_| Error::GameQuit)?;
        let res = rx.recv().map_err(|_| Error::GameQuit)?;
        Ok(res)
    })
    .await
    .unwrap()?;
    Ok(web::Json(res))
}

#[derive(Serialize, Deserialize, Default)]
struct PlaceRequest {
    index: usize,
}
impl PlaceRequest {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
    pub fn set_index(&mut self, index: usize) {
        self.index = index
    }
}

#[derive(Debug)]
pub enum Error {
    InsertError(InsertError),
    GameQuit,
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsertError(e) => write!(f, "{}", e),
            Self::GameQuit => write!(f, "Game quit"),
        }
    }
}
impl error::ResponseError for Error {}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc::channel};

    use actix_web::{
        App, test,
        web::{self, Data},
    };
    use common::tic_tac_toe::{board::Board, common::Message, spawn_game_tokio};

    use crate::{PlaceRequest, State, board, place};

    #[actix_web::test]
    async fn test_board() {
        let (handle, tx) = spawn_game_tokio();

        let app = test::init_service(
            App::new()
                .app_data(Data::new(State::new(tx)))
                .service(board),
        )
        .await;
        let req = test::TestRequest::get().uri("/board").to_request();
        let resp: Board = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp, Board::default());
    }
}
