use std::{collections::HashMap, sync::Arc};

use axum::{
  Extension, Json,
  extract::{
    Path, State, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  http::{StatusCode, response},
  response::{IntoResponse, Response},
};
use futures::{
  SinkExt,
  stream::{self, StreamExt},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use snowflaked::Generator;
use tokio::sync::broadcast;

use crate::{
  AppState, RoomState,
  controllers::{
    games::minesweeper::{CONSTANTS, CreateGameResponse, GameParameters},
    user::ErrorMessage,
  },
  models::{
    battleship::{self, Battleship, CreateShip, UpdateGame},
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

#[axum::debug_handler]
pub async fn games(
  ws: WebSocketUpgrade,
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Response {
  ws.on_upgrade(|socket| handle_game_socket(socket, state, user))
}

#[derive(Serialize, Deserialize, Debug)]
struct SocketMessage<T> {
  r#type: String,
  data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JoinRoomPackets {
  id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Packets {
  #[serde(rename = "join-game")]
  JoinGame { data: JoinRoomPackets },
}

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>, user: user::User) {
  // let _ = socket.send(Message::Text(format!("connected"))).await;

  let (mut sender, mut receiver) = socket.split();

  let mut tx = None::<broadcast::Sender<String>>;
  let mut roomid = String::new();
  // let mut channel = String::new();

  while let Some(Ok(msg)) = receiver.next().await {
    if let Message::Text(msg) = msg {
      println!("msg, {:?}", msg);

      #[derive(Deserialize)]
      struct Connect {
        roomid: String,
      }

      let connect: Connect = match serde_json::from_str(&msg) {
        Ok(connect) => connect,
        Err(error) => {
          tracing::error!(%error);
          let _ = sender
            .send(Message::Text(String::from(
              "Failed to parse connect message",
            )))
            .await;
          break;
        }
      };
      // when user joins room, just add another entry to the state, just like below and then send a emit on the client, see if message is only recieved in that socket.

      {
        let mut rooms = state.rooms.lock().unwrap();

        roomid = connect.roomid.clone();
        let room = rooms.entry(connect.roomid).or_insert_with(RoomState::new);

        tx = Some(room.tx.clone());

        if !room.user_set.contains(&user.id) {
          room.user_set.insert(user.id);
        }
      }

      if tx.is_some() {
        break;
      } else {
        let _ = sender
          .send(Message::Text(String::from("Username already in room.")))
          .await;

        return;
      }
    }
  }

  let tx = tx.unwrap();

  let mut rx: broadcast::Receiver<String> = tx.subscribe();

  // Send joined message to all subscribers.
  let msg = format!("{} joined {}.", user.username, roomid);
  tracing::debug!("{}", msg);
  let _ = tx.send(msg);

  let send_roomid = roomid.clone();
  let mut send_task = tokio::spawn(async move {
    while let Ok(msg) = rx.recv().await {
      println!("[{}:send]: {:?}", send_roomid, msg);
      // In any websocket error, break loop.
      if sender.send(Message::Text(msg)).await.is_err() {
        break;
      }
    }
  });

  let mut recv_task = {
    // Clone things we want to pass to the receiving task.
    let tx: broadcast::Sender<String> = tx.clone();
    let name = user.username.clone();
    let roomid = roomid.clone();
    let recv_pool = state.pool.clone();

    // This task will receive messages from client and send them to broadcast subscribers.
    tokio::spawn(async move {
      while let Some(Ok(Message::Text(text))) = receiver.next().await {
        println!("[{}:recv]: {:?}", roomid, text);

        let packets: Packets = match serde_json::from_str(&text) {
          Ok(msg) => msg,
          Err(er) => {
            println!("{:?}", er);
            break;
          }
        };

        println!("packets {:?}", packets);

        match packets {
          Packets::JoinGame { data } => {}
          _ => {
            let _ = tx.send(format!("{}: {}", name, text));
          }
        }
      }
    })
  };

  tokio::select! {
  _ = (&mut send_task) => recv_task.abort(),
  _ = (&mut recv_task) => send_task.abort(),
  };

  let msg = format!("{} left {}.", user.username, roomid);
  tracing::debug!("{}", msg);
  let _ = tx.send(msg);
  let mut rooms = state.rooms.lock().unwrap();

  rooms.get_mut(&roomid).unwrap().user_set.remove(&user.id);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Games {
  #[serde(rename = "get-games")]
  GetGames,
  #[serde(rename = "game-created")]
  GameCreated,
}

pub async fn handle_game_socket(socket: WebSocket, state: Arc<AppState>, user: user::User) {
  // let _ = socket.send(Message::Text(format!("connected"))).await;

  let (mut sender, mut receiver) = socket.split();

  let mut tx = None::<broadcast::Sender<String>>;
  let mut roomid = String::new();
  // let mut channel = String::new();

  while let Some(Ok(msg)) = receiver.next().await {
    if let Message::Text(msg) = msg {
      println!("msg, {:?}", msg);

      #[derive(Deserialize)]
      struct Connect {
        roomid: String,
      }

      let connect: Connect = match serde_json::from_str(&msg) {
        Ok(connect) => connect,
        Err(error) => {
          tracing::error!(%error);
          let _ = sender
            .send(Message::Text(String::from(
              "Failed to parse connect message",
            )))
            .await;
          break;
        }
      };
      // when user joins room, just add another entry to the state, just like below and then send a emit on the client, see if message is only recieved in that socket.

      {
        let mut rooms = state.rooms.lock().unwrap();

        roomid = connect.roomid.clone();
        let room = rooms.entry(connect.roomid).or_insert_with(RoomState::new);

        tx = Some(room.tx.clone());

        if !room.user_set.contains(&user.id) {
          room.user_set.insert(user.id);
        }
      }

      if tx.is_some() {
        break;
      } else {
        let _ = sender
          .send(Message::Text(String::from("Username already in room.")))
          .await;

        return;
      }
    }
  }

  let tx = tx.unwrap();

  let mut rx: broadcast::Receiver<String> = tx.subscribe();

  // Send joined message to all subscribers.
  let msg = format!("{} joined {}.", user.username, roomid);
  tracing::debug!("{}", msg);
  let _ = tx.send(msg);

  let send_roomid = roomid.clone();
  let mut send_task = tokio::spawn(async move {
    while let Ok(msg) = rx.recv().await {
      println!("[{}:send]: {:?}", send_roomid, msg);
      // In any websocket error, break loop.
      if sender.send(Message::Text(msg)).await.is_err() {
        break;
      }
    }
  });

  let mut recv_task = {
    // Clone things we want to pass to the receiving task.
    let tx: broadcast::Sender<String> = tx.clone();
    let name = user.username.clone();
    let roomid = roomid.clone();
    let recv_pool = state.pool.clone();

    // This task will receive messages from client and send them to broadcast subscribers.
    tokio::spawn(async move {
      while let Some(Ok(Message::Text(text))) = receiver.next().await {
        println!("[{}:recv]: {:?}", roomid, text);

        let packets: Games = match serde_json::from_str(&text) {
          Ok(msg) => msg,
          Err(er) => {
            println!("{:?}", er);
            break;
          }
        };

        println!("packets {:?}", packets);

        match packets {
          Games::GetGames => {
            let games =
              battleship::get_games_by_state(&recv_pool, CONSTANTS.awaiting.to_string()).await;

            let games = match games {
              Ok(games) => match games {
                Some(games) => games,
                None => Vec::new(),
              },
              Err(_) => Vec::new(),
            };

            let mut response_game: Vec<BattleshipsGame> = Vec::new();

            for game in games {
              let belongs_to_username = user::user_exist_by_id(&recv_pool, &game.belongs_to)
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
                ships: HashMap::new(),
                turn: game.turn == user.id,
              })
            }
            let _ = tx.send(
              serde_json::to_string(&SocketMessage {
                r#type: "get-games".to_string(),
                data: Some(response_game),
              })
              .unwrap(),
            );
          }
          _ => {
            let _ = tx.send(format!("{}: {}", name, text));
          }
        }
      }
    })
  };

  tokio::select! {
  _ = (&mut send_task) => recv_task.abort(),
  _ = (&mut recv_task) => send_task.abort(),
  };

  let msg = format!("{} left {}.", user.username, roomid);
  tracing::debug!("{}", msg);
  let _ = tx.send(msg);
  let mut rooms = state.rooms.lock().unwrap();

  rooms.get_mut(&roomid).unwrap().user_set.remove(&user.id);
}

#[derive(Serialize, Deserialize)]
pub struct CreateGame {
  stake: i64,
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

  {
    let mut rooms = state.rooms.lock().unwrap();
    let roomid = "games".to_string();
    println!("{:?}", rooms.get_mut(&roomid).unwrap());
    let _ = rooms.get_mut(&roomid).unwrap().tx.send(
      serde_json::to_string(&SocketMessage::<String> {
        r#type: "game-created".to_string(),
        data: None,
      })
      .unwrap(),
    );
  }

  Ok((
    StatusCode::OK,
    Json(CreateGameResponse {
      gameid: game.id.to_string(),
    }),
  ))
}

#[derive(Serialize, Deserialize)]
pub struct JoinGame {
  pub gameid: String,
}

pub async fn join_game(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Json(input): Json<JoinGame>,
) -> Result<(StatusCode, Json<CreateGameResponse>), (StatusCode, Json<ErrorMessage>)> {
  let game_id = input.gameid.parse::<i64>().map_err(|_| {
    (
      StatusCode::FORBIDDEN,
      Json(ErrorMessage {
        msg: "id must be a valid number".to_string(),
      }),
    )
  })?;
  let game = battleship::get_game_by_id(&state.pool, game_id)
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

  {
    let mut rooms = state.rooms.lock().unwrap();
    let _ = rooms.get_mut(&game_id.to_string()).unwrap().tx.send(
      serde_json::to_string(&SocketMessage {
        r#type: "user-joined".to_string(),
        data: Some(CreateGameResponse {
          gameid: game.id.to_string(),
        }),
      })
      .unwrap(),
    );
  }

  Ok((
    StatusCode::OK,
    Json(CreateGameResponse {
      gameid: game.id.to_string(),
    }),
  ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BattleshipOpponent {
  ready: bool,
  ships: Option<HashMap<String, Vec<i64>>>,
  clicks: Vec<i64>,
}

fn covert_ships(ships: &Vec<battleship::BattleshipShips>) -> HashMap<String, Vec<i64>> {
  let mut converted_ships: HashMap<String, Vec<i64>> = HashMap::new();
  for ship in ships {
    let steps = if ship.direction == CONSTANTS.right.to_string() {
      1
    } else {
      8
    };
    converted_ships.insert(
      ship.size.to_string(),
      (ship.position..(ship.position + ship.size * steps))
        .step_by(steps as usize)
        .collect(),
    );
  }
  converted_ships
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BattleshipsGame {
  pub id: String,
  pub belongs_to: String,
  pub state: String,
  pub winner: Option<String>,
  pub turn: bool,
  pub stake: i64,
  pub pool: i64,
  pub ready: bool,
  pub opponent: Option<BattleshipOpponent>,
  pub ships: HashMap<String, Vec<i64>>,
  pub clicks: Vec<i64>,
}

pub async fn get_latest(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, Json<BattleshipsGame>), (StatusCode, Json<ErrorMessage>)> {
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
      Json(BattleshipsGame {
        id: game.id.to_string(),
        belongs_to: belongs_to_username,
        state: game.state,
        stake: game.stake.0,
        pool: game.pool.0,
        ready: false,
        winner: None,
        opponent: None,
        clicks: Vec::new(),
        ships: HashMap::new(),
        turn: game.turn == user.id,
      }),
    ));
  }

  let opponent_id = if user.id == game.belongs_to {
    game.opponent.unwrap()
  } else {
    game.belongs_to
  };

  let ships = battleship::get_ships_by_user_and_game(&state.pool, user.id, game.id)
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
    ))?;

  let opponent_ships = battleship::get_ships_by_user_and_game(&state.pool, opponent_id, game.id)
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
    ))?;

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
      Json(BattleshipsGame {
        id: game.id.to_string(),
        state: game.state,
        belongs_to: belongs_to_username,
        stake: game.stake.0,
        pool: game.pool.0,
        ready: ships.len() as i64 == CONSTANTS.number_of_ships,
        winner: Some(winner.username),
        opponent: Some(BattleshipOpponent {
          ready: opponent_ships.len() as i64 == CONSTANTS.number_of_ships,
          ships: Some(covert_ships(&opponent_ships)),
          clicks: match opponent_clicks {
            Some(value) => value.iter().map(|click| click.position).collect(),
            None => Vec::new(),
          },
        }),
        clicks: match clicks {
          Some(value) => value.iter().map(|click| click.position).collect(),
          None => Vec::new(),
        },
        ships: covert_ships(&ships),
        turn: game.turn == user.id,
      }),
    ));
  }

  // STATE: game is playing
  Ok((
    StatusCode::OK,
    Json(BattleshipsGame {
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
      ships: covert_ships(&ships),
      turn: game.turn == user.id,
    }),
  ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GamePlacement {
  pub gameid: String,
  pub position: HashMap<i64, Vec<i64>>,
}

pub async fn confirm_placement(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Json(input): Json<GamePlacement>,
) -> Result<(StatusCode, Json<String>), (StatusCode, Json<ErrorMessage>)> {
  let game_id = input.gameid.parse::<i64>().map_err(|_| {
    (
      StatusCode::FORBIDDEN,
      Json(ErrorMessage {
        msg: "id must be a valid number".to_string(),
      }),
    )
  })?;
  let game = battleship::get_game_by_user_and_game_id(&state.pool, user.id, game_id)
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

  if ships.len() as i64 == CONSTANTS.number_of_ships {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorMessage {
        msg: "User has already placed their ships".to_string(),
      }),
    ));
  }

  let mut ships: Vec<CreateShip> = Vec::new();
  let mut generator = Generator::new(0);

  for (key, value) in input.position {
    ships.push(CreateShip {
      id: generator.generate(),
      belongs_to: game.id,
      position: value[0],
      placed_by: user.id,
      size: key,
      direction: if value.len() as i64 > 1 {
        if value[1] - value[0] == 1 {
          CONSTANTS.right.to_string()
        } else {
          CONSTANTS.down.to_string()
        }
      } else {
        CONSTANTS.down.to_string()
      },
    })
  }

  let _ = battleship::create_ships(&state.pool, &ships)
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

  let opponent_id = if user.id == game.belongs_to {
    game.opponent.unwrap()
  } else {
    game.belongs_to
  };

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

  {
    let mut rooms = state.rooms.lock().unwrap();
    let _ = rooms.get_mut(&game_id.to_string()).unwrap().tx.send(
      serde_json::to_string(&SocketMessage {
        r#type: "user-ready".to_string(),
        data: Some(CreateGameResponse {
          gameid: game.id.to_string(),
        }),
      })
      .unwrap(),
    );
  }

  if opponent_ships.len() as i64 == CONSTANTS.number_of_ships {
    let _ = battleship::update_game(
      &state.pool,
      game.id,
      &UpdateGame {
        state: Some(CONSTANTS.ongoing.to_string()),
        pool: None,
        turn: None,
        opponent: None,
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
    return Ok((StatusCode::OK, Json("Game start".to_string())));
  }

  Ok((StatusCode::OK, Json("One user ready".to_string())))
}

pub async fn get(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Path(params): Path<GameParameters>,
) -> Result<(StatusCode, Json<BattleshipsGame>), (StatusCode, Json<ErrorMessage>)> {
  let id = params
    .id
    .ok_or((
      StatusCode::FORBIDDEN,
      Json(ErrorMessage {
        msg: "Game ID must be provided".to_string(),
      }),
    ))?
    .parse::<i64>()
    .map_err(|_| {
      (
        StatusCode::FORBIDDEN,
        Json(ErrorMessage {
          msg: "id must be a valid number".to_string(),
        }),
      )
    })?;

  let game = battleship::get_game_by_id(&state.pool, id)
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
        msg: "Game does not exist".to_string(),
      }),
    ))?;

  if game.belongs_to != user.id && game.opponent.is_some_and(|op_id| op_id != user.id) {
    return Err((
      StatusCode::UNAUTHORIZED,
      Json(ErrorMessage {
        msg: "Don't have access to this game".to_string(),
      }),
    ));
  }

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
      Json(BattleshipsGame {
        id: game.id.to_string(),
        belongs_to: belongs_to_username,
        state: game.state,
        stake: game.stake.0,
        pool: game.pool.0,
        ready: false,
        winner: None,
        opponent: None,
        clicks: Vec::new(),
        ships: HashMap::new(),
        turn: game.turn == user.id,
      }),
    ));
  }

  let opponent_id = if user.id == game.belongs_to {
    game.opponent.unwrap()
  } else {
    game.belongs_to
  };

  let ships = battleship::get_ships_by_user_and_game(&state.pool, user.id, game.id)
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
    ))?;

  let opponent_ships = battleship::get_ships_by_user_and_game(&state.pool, opponent_id, game.id)
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
    ))?;

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
      Json(BattleshipsGame {
        id: game.id.to_string(),
        state: game.state,
        belongs_to: belongs_to_username,
        stake: game.stake.0,
        pool: game.pool.0,
        ready: ships.len() as i64 == CONSTANTS.number_of_ships,
        winner: Some(winner.username),
        opponent: Some(BattleshipOpponent {
          ready: opponent_ships.len() as i64 == CONSTANTS.number_of_ships,
          ships: Some(covert_ships(&opponent_ships)),
          clicks: match opponent_clicks {
            Some(value) => value.iter().map(|click| click.position).collect(),
            None => Vec::new(),
          },
        }),
        clicks: match clicks {
          Some(value) => value.iter().map(|click| click.position).collect(),
          None => Vec::new(),
        },
        ships: covert_ships(&ships),
        turn: game.turn == user.id,
      }),
    ));
  }

  // STATE: game is playing
  Ok((
    StatusCode::OK,
    Json(BattleshipsGame {
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
      ships: covert_ships(&ships),
      turn: game.turn == user.id,
    }),
  ))
}
