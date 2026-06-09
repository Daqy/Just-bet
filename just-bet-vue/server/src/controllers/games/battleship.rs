use std::sync::Arc;

use axum::{
  Extension, Json,
  extract::{
    State, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use snowflaked::Generator;

use crate::{
  AppState,
  controllers::{
    games::minesweeper::{CONSTANTS, CreateGameResponse},
    user::ErrorMessage,
  },
  models::{
    battleship::{self, Battleship},
    user,
  },
};
use diesel::data_types::PgMoney;

#[axum::debug_handler]
pub async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
  ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(serde::Deserialize, Debug)]
struct SocketMessage {
  r#type: String,
  data: Option<serde_json::Value>,
}

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
  // let _ = socket.send(Message::Text(format!("connected"))).await;

  while let Some(msg) = socket.recv().await {
    let msg = if let Ok(msg) = msg {
      println!("msg, {:?}", msg);

      let message: Option<Message> = match msg {
        Message::Text(text) => {
          let socket_message: SocketMessage = serde_json::from_str(&text).unwrap();

          let response = if socket_message.r#type == "connected" {
            Some(Message::Text("User connected".to_string()))
          } else if socket_message.r#type == "get_games" {
            // Some()
            None
          } else {
            None
          };
          // if socket_message.r#type == "connected" {
          //   return Some(Message::Text("User connected".to_string()));
          // }
          response
        }
        _ => None,
      };

      match message {
        Some(message) => message,
        None => Message::Text("Socket doesn't exist".to_string()),
      }
      // msg
    } else {
      // client disconnected
      return;
    };

    if socket.send(msg).await.is_err() {
      // client disconnected
      return;
    }
    // if let Ok(msg) = msg {

    //   match msg {
    //     Message::Text(text) => {
    //       println!("{:?}", socket_message);

    //         // let result = socket.send(Message::Text(format!("User connected"))).await;
    //         // msg
    //       }
    //       // let result = socket
    //       //   .send(Message::Text(format!("Echo back text: {}", text).into()))
    //       //   .await;
    //       msg
    //     }
    //     _ => {}
    //   }

    //   // msg
    // } else {
    //   // client disconnected
    //   println!("Client disconnected");
    //   return;
    // };

    // let _ = socket.send(Message::Text(format!("testing"))).await;

    // if socket.send(msg.unwrap()).await.is_err() {
    //   // client disconnected
    //   println!("Client disconnected 1");
    //   return;
    // }
  }
}

#[derive(Serialize, Deserialize)]
pub struct CreateGame {
  stake: i64,
}
impl IntoResponse for CreateGame {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

pub async fn create(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Json(input): Json<CreateGame>,
) -> Result<(StatusCode, Json<CreateGameResponse>), (StatusCode, Json<ErrorMessage>)> {
  let latest_game = battleship::get_game_by_user_id(&state.pool, user.id)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?;

  match latest_game {
    Some(game) => {
      if game.state != CONSTANTS.done {
        return Err((
          StatusCode::NOT_FOUND,
          Json(ErrorMessage {
            msg: "User is already in a game".to_string(),
          }),
        ));
      }
    }
    None => {}
  }

  let mut generator = Generator::new(0);

  let game = battleship::create_game(
    &state.pool,
    &Battleship {
      id: generator.generate(),
      belongs_to: user.id,
      state: CONSTANTS.awaiting.to_string(),
      opponent: None,
      winner: None,
      turn: user.id,
      stake: PgMoney(input.stake * 100),
      pool: PgMoney(input.stake * 100),
    },
  )
  .await
  .map_err(|_| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorMessage {
        msg: "Something really went wrong".to_string(),
      }),
    )
  })?
  .pop()
  .unwrap();

  user::update_balance(
    &state.pool,
    user.id,
    Some(user.balance - PgMoney(input.stake * 100)),
  )
  .await
  .map_err(|_| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorMessage {
        msg: "Something really went wrong".to_string(),
      }),
    )
  })?;

  Ok((
    StatusCode::OK,
    Json(CreateGameResponse {
      gameid: game.id.to_string(),
    }),
  ))
}
