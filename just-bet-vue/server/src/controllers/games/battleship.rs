use std::{any::Any, sync::Arc, vec};

use axum::{
  Extension, Json,
  extract::{
    State, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  http::{StatusCode, response},
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
    battleship::{self, Battleship, UpdateGame},
    user,
  },
};
use diesel::data_types::PgMoney;

#[axum::debug_handler]
pub async fn handler(
  ws: WebSocketUpgrade,
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Response {
  ws.on_upgrade(|socket| handle_socket(socket, state, user))
}

#[derive(Serialize, Deserialize, Debug)]
struct SocketMessage<T> {
  r#type: String,
  data: Option<T>,
}

pub async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>, user: user::User) {
  // let _ = socket.send(Message::Text(format!("connected"))).await;

  while let Some(msg) = socket.recv().await {
    let msg = if let Ok(msg) = msg {
      println!("msg, {:?}", msg);

      let message: Option<Message> = match msg {
        Message::Text(text) => {
          let socket_message: SocketMessage<String> = serde_json::from_str(&text).unwrap();

          let response = if socket_message.r#type == "connected" {
            Some(Message::Text(
              serde_json::to_string(&SocketMessage::<String> {
                r#type: "connect".to_string(),
                data: None,
              })
              .unwrap(),
            ))
          } else if socket_message.r#type == "get-games" {
            let games =
              battleship::get_games_by_state(&state.pool, CONSTANTS.awaiting.to_string()).await;

            let games = match games {
              Ok(games) => match games {
                Some(games) => games,
                None => Vec::new(),
              },
              Err(_) => Vec::new(),
            };

            let mut response_game: Vec<BattleshipsGame> = Vec::new();

            for game in games {
              let belongs_to_username = user::user_exist_by_id(&state.pool, &game.belongs_to)
                .await
                .unwrap()
                .unwrap()
                .username;

              response_game.push(BattleshipsGame {
                id: game.id.to_string(),
                state: game.state.clone(),
                belongs_to: belongs_to_username,
                stake: game.stake.0,
                pool: game.pool.0,
                ready: false,
                winner: None,
                opponent: None,
                clicks: Vec::new(),
                ships: Vec::new(),
                turn: game.turn.to_string(),
              })
            }

            Some(Message::Text(
              serde_json::to_string(&SocketMessage {
                r#type: "get-games".to_string(),
                data: Some(response_game),
              })
              .unwrap(),
            ))
          } else {
            None
          };

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

#[derive(Serialize, Deserialize)]
pub struct JoinGame {
  pub gameid: i64,
}
impl IntoResponse for JoinGame {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

pub async fn join_game(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Json(input): Json<JoinGame>,
) -> Result<(StatusCode, Json<CreateGameResponse>), (StatusCode, Json<ErrorMessage>)> {
  let game = battleship::get_game_by_id(&state.pool, input.gameid)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?
    .ok_or((
      StatusCode::NOT_FOUND,
      Json(ErrorMessage {
        msg: "Unable to find game".to_string(),
      }),
    ))?;

  if game.opponent != None {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorMessage {
        msg: "Game already has an opponent".to_string(),
      }),
    ));
  }

  let rand = rand::random_range(1..2);

  let game = battleship::update_game(
    &state.pool,
    game.id,
    &UpdateGame {
      state: Some(CONSTANTS.prep.to_string()),
      pool: Some(PgMoney(game.stake.0 * 2)),
      turn: if rand == 1 {
        Some(user.id)
      } else {
        Some(game.belongs_to)
      },
      opponent: Some(user.id),
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
  })?;

  user::update_balance(&state.pool, user.id, Some(user.balance - game.stake))
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

#[derive(Serialize, Deserialize)]
pub struct BattleshipOpponent {
  ready: bool,
  ships: Option<Vec<i64>>,
  clicks: Vec<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct BattleshipsGame {
  pub id: String,
  pub belongs_to: String,
  pub state: String,
  pub winner: Option<String>,
  pub turn: String,
  pub stake: i64,
  pub pool: i64,
  pub ready: bool,
  pub opponent: Option<BattleshipOpponent>,
  pub ships: Vec<i64>,
  pub clicks: Vec<i64>,
}

impl IntoResponse for BattleshipsGame {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

pub async fn get_latest(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, BattleshipsGame), (StatusCode, Json<ErrorMessage>)> {
  let game = battleship::get_game_by_user_id(&state.pool, user.id)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?
    .ok_or((
      StatusCode::OK,
      Json(ErrorMessage {
        msg: "User doesn't have any games".to_string(),
      }),
    ))?;

  let belongs_to_username = user::user_exist_by_id(&state.pool, &game.belongs_to)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?
    .ok_or((
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorMessage {
        msg: "User no longer exist".to_string(),
      }),
    ))?
    .username;

  if game.state == CONSTANTS.awaiting.to_string() {
    // STATE: waiting for player to join
    return Ok((
      StatusCode::OK,
      BattleshipsGame {
        id: game.id.to_string(),
        belongs_to: belongs_to_username,
        state: game.state,
        stake: game.stake.0,
        pool: game.pool.0,
        ready: false,
        winner: None,
        opponent: None,
        clicks: Vec::new(),
        ships: Vec::new(),
        turn: game.turn.to_string(),
      },
    ));
  }

  let opponent_id = if user.id == game.belongs_to {
    game.opponent.unwrap()
  } else {
    game.belongs_to
  };

  let ships: Vec<i64> = battleship::get_ships_by_user_and_game(&state.pool, user.id, game.id)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?
    .ok_or((
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorMessage {
        msg: "Game wasn't craeted correctly".to_string(),
      }),
    ))?
    .iter()
    .map(|ship| ship.position)
    .collect();

  let opponent_ships: Vec<i64> =
    battleship::get_ships_by_user_and_game(&state.pool, opponent_id, game.id)
      .await
      .map_err(|_| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(ErrorMessage {
            msg: "Something really went wrong".to_string(),
          }),
        )
      })?
      .ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Game wasn't craeted correctly".to_string(),
        }),
      ))?
      .iter()
      .map(|ship| ship.position)
      .collect();

  let clicks = battleship::get_clicks_by_user_and_game(&state.pool, user.id, game.id)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?;

  let opponent_clicks = battleship::get_clicks_by_user_and_game(&state.pool, opponent_id, game.id)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?;

  if game.state == CONSTANTS.done {
    let winner = user::user_exist_by_id(&state.pool, &game.winner.unwrap())
      .await
      .map_err(|_| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(ErrorMessage {
            msg: "Something really went wrong".to_string(),
          }),
        )
      })?
      .ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "User no longer exist".to_string(),
        }),
      ))?;

    // STATE: game is finished
    return Ok((
      StatusCode::OK,
      BattleshipsGame {
        id: game.id.to_string(),
        state: game.state,
        belongs_to: belongs_to_username,
        stake: game.stake.0,
        pool: game.pool.0,
        ready: ships.len() as i64 == CONSTANTS.number_of_ships,
        winner: Some(winner.username),
        opponent: Some(BattleshipOpponent {
          ready: opponent_ships.len() as i64 == CONSTANTS.number_of_ships,
          ships: Some(opponent_ships),
          clicks: match opponent_clicks {
            Some(value) => value.iter().map(|click| click.position).collect(),
            None => Vec::new(),
          },
        }),
        clicks: match clicks {
          Some(value) => value.iter().map(|click| click.position).collect(),
          None => Vec::new(),
        },
        ships: ships,
        turn: game.turn.to_string(),
      },
    ));
  }

  // STATE: game is playing
  Ok((
    StatusCode::OK,
    BattleshipsGame {
      id: game.id.to_string(),
      state: game.state,
      belongs_to: belongs_to_username,
      stake: game.stake.0,
      pool: game.pool.0,
      ready: ships.len() as i64 == CONSTANTS.number_of_ships,
      winner: None,
      opponent: Some(BattleshipOpponent {
        ready: opponent_ships.len() as i64 == CONSTANTS.number_of_ships,
        ships: None,
        clicks: match opponent_clicks {
          Some(value) => value.iter().map(|click| click.position).collect(),
          None => Vec::new(),
        },
      }),
      clicks: match clicks {
        Some(value) => value.iter().map(|click| click.position).collect(),
        None => Vec::new(),
      },
      ships: ships,
      turn: game.turn.to_string(),
    },
  ))
}
