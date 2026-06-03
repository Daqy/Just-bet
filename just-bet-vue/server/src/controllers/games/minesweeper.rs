use std::sync::Arc;

use axum::{
  Extension, Json,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use diesel::data_types::PgMoney;
use serde::{Deserialize, Serialize};
use serde_json::json;
use snowflaked::Generator;

use crate::{
  AppState,
  controllers::user::ErrorMessage,
  models::{
    minesweeper::{self, NewBomb},
    user,
  },
};

#[derive(Serialize, Deserialize)]
pub struct MinesweeperBombs {
  count: i64,
  position: Vec<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct MinesweeperClick {
  position: i64,
  earned: i64,
}
#[derive(Serialize, Deserialize)]
pub struct MinesweeperGame {
  pub state: String,
  pub result: String,
  pub stake: i64,
  pub pool: i64,
  pub bomb: MinesweeperBombs,
  pub clicks: Vec<MinesweeperClick>,
  pub size: i64,
}

impl IntoResponse for MinesweeperGame {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

#[axum::debug_handler]
pub async fn get(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, MinesweeperGame), (StatusCode, Json<ErrorMessage>)> {
  let game = minesweeper::get_game_by_user_id(&state.pool, &user.id)
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

  let bombs: Vec<i64> = minesweeper::get_bombs_by_game(&state.pool, &game)
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
    .map(|bomb| bomb.position)
    .collect();

  let response_bombs: MinesweeperBombs = if game.state == CONSTANTS.done {
    MinesweeperBombs {
      count: bombs.len() as i64,
      position: bombs,
    }
  } else {
    MinesweeperBombs {
      count: bombs.len() as i64,
      position: Vec::new(),
    }
  };

  let clicks = minesweeper::get_clicks_by_game(&state.pool, &game)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?;

  match clicks {
    Some(value) => Ok((
      StatusCode::OK,
      MinesweeperGame {
        state: game.state,
        result: game.result,
        stake: game.stake.0,
        pool: game.pool.0,
        bomb: response_bombs,
        size: 25,
        clicks: value
          .iter()
          .map(|click| MinesweeperClick {
            earned: click.earned.0,
            position: click.position,
          })
          .collect(),
      },
    )),
    None => Ok((
      StatusCode::OK,
      MinesweeperGame {
        state: game.state,
        result: game.result,
        size: 25,
        stake: game.stake.0,
        pool: game.pool.0,
        bomb: response_bombs,
        clicks: Vec::new(),
      },
    )),
  }
}

#[derive(Serialize, Deserialize)]
pub struct CreateGame {
  stake: i64,
  bomb_count: i64,
}
impl IntoResponse for CreateGame {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

struct Constants<'a> {
  lost: &'a str,
  claimed: &'a str,
  done: &'a str,
  ongoing: &'a str,
  awaiting: &'a str,
  prep: &'a str,
}

const CONSTANTS: Constants = {
  Constants {
    lost: "lost",
    claimed: "claimed",
    done: "done",
    ongoing: "ongoing",
    awaiting: "awaiting",
    prep: "prep",
  }
};

#[derive(Serialize, Deserialize)]
pub struct CreateGameResponse {
  pub gameid: i64,
}
impl IntoResponse for CreateGameResponse {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}
pub async fn create(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Json(input): Json<CreateGame>,
) -> Result<(StatusCode, Json<CreateGameResponse>), (StatusCode, Json<ErrorMessage>)> {
  let latest_game = minesweeper::get_game_by_user_id(&state.pool, &user.id)
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
      if game.state == CONSTANTS.ongoing.to_string() {
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

  let game = minesweeper::create_game(
    &state.pool,
    &minesweeper::Minesweeper {
      id: generator.generate(),
      belongs_to: user.id,
      state: CONSTANTS.ongoing.to_string(),
      result: "no winner".to_string(),
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

  let mut bombs: Vec<NewBomb> = Vec::new();

  while (bombs.len() as i64) < input.bomb_count {
    let rand = rand::random_range(1..25);
    if bombs.iter().any(|&bomb| bomb.position == rand) {
      continue;
    }
    bombs.push(NewBomb {
      id: generator.generate(),
      belongs_to: game.id,
      position: rand,
    });
  }

  minesweeper::create_bomb_for_game(&state.pool, &bombs)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?;

  Ok((StatusCode::OK, Json(CreateGameResponse { gameid: game.id })))
}
