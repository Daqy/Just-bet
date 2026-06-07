use std::sync::Arc;

use axum::{
  Extension, Json,
  extract::{Path, Query, State},
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
    minesweeper::{self, Click, NewBomb, UpdateGame},
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
  pub id: String,
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
pub async fn get_latest(
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

  Ok((
    StatusCode::OK,
    MinesweeperGame {
      id: game.id.to_string(),
      state: game.state,
      result: game.result,
      stake: game.stake.0,
      pool: game.pool.0,
      bomb: response_bombs,
      size: CONSTANTS.game_size,
      clicks: match clicks {
        Some(value) => value
          .iter()
          .map(|click| MinesweeperClick {
            earned: click.earned.0,
            position: click.position,
          })
          .collect(),
        None => Vec::new(),
      },
    },
  ))
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

pub struct Constants<'a> {
  pub lost: &'a str,
  pub claimed: &'a str,
  pub done: &'a str,
  pub ongoing: &'a str,
  pub awaiting: &'a str,
  pub prep: &'a str,
  pub game_size: i64,
  pub claim_amount: PgMoney,
}

pub const CONSTANTS: Constants = {
  Constants {
    lost: "lost",
    claimed: "claimed",
    done: "done",
    ongoing: "ongoing",
    awaiting: "awaiting",
    prep: "prep",
    game_size: 25,
    claim_amount: PgMoney(5000),
  }
};

#[derive(Serialize, Deserialize)]
pub struct CreateGameResponse {
  pub gameid: String,
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
    let rand = rand::random_range(1..CONSTANTS.game_size);
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

#[derive(Deserialize, Debug)]
pub struct GameParameters {
  pub id: Option<i64>,
}

pub async fn get(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Path(params): Path<GameParameters>,
) -> Result<(StatusCode, MinesweeperGame), (StatusCode, Json<ErrorMessage>)> {
  // for (key, value) in &params {}

  let id = params.id.ok_or((
    StatusCode::FORBIDDEN,
    Json(ErrorMessage {
      msg: "Game ID must be provided".to_string(),
    }),
  ))?;

  let game = minesweeper::get_game_by_id(&state.pool, id)
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

  if game.belongs_to != user.id {
    return Err((
      StatusCode::UNAUTHORIZED,
      Json(ErrorMessage {
        msg: "Don't have access to this game".to_string(),
      }),
    ));
  }

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

  Ok((
    StatusCode::OK,
    MinesweeperGame {
      id: game.id.to_string(),
      state: game.state,
      result: game.result,
      stake: game.stake.0,
      pool: game.pool.0,
      bomb: response_bombs,
      size: CONSTANTS.game_size,
      clicks: match clicks {
        Some(value) => value
          .iter()
          .map(|click| MinesweeperClick {
            earned: click.earned.0,
            position: click.position,
          })
          .collect(),
        None => Vec::new(),
      },
    },
  ))
}

#[axum::debug_handler]
pub async fn get_games(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, Json<Vec<MinesweeperGame>>), (StatusCode, Json<ErrorMessage>)> {
  let games = minesweeper::get_games_by_user_id(&state.pool, user.id)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?;

  println!("{:?}", games);
  match games {
    Some(games) => {
      let mut response_games: Vec<MinesweeperGame> = Vec::new();
      for game in games {
        if game.state != CONSTANTS.done.to_string() {
          continue;
        }
        let clicks = minesweeper::get_clicks_by_game(&state.pool, &game)
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
            StatusCode::FORBIDDEN,
            Json(ErrorMessage {
              msg: "no clicks found".to_string(),
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

        response_games.push(MinesweeperGame {
          id: game.id.to_string(),
          state: game.state,
          result: game.result,
          stake: game.stake.0,
          pool: game.pool.0,
          bomb: MinesweeperBombs {
            count: bombs.len() as i64,
            position: bombs,
          },
          size: CONSTANTS.game_size,
          clicks: clicks
            .iter()
            .map(|click| MinesweeperClick {
              earned: click.earned.0,
              position: click.position,
            })
            .collect(),
        })
      }

      Ok((StatusCode::OK, Json(response_games)))
    }
    None => Ok((StatusCode::OK, Json(Vec::new()))),
  }
}

#[derive(Deserialize, Debug)]
pub struct ClickQuery {
  pub click_position: Option<i64>,
}
pub async fn click(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Path(params): Path<GameParameters>,
  query: Query<ClickQuery>,
) -> Result<(StatusCode, MinesweeperGame), (StatusCode, Json<ErrorMessage>)> {
  let id = params.id.ok_or((
    StatusCode::FORBIDDEN,
    Json(ErrorMessage {
      msg: "Game ID must be provided".to_string(),
    }),
  ))?;

  let click_position = query.click_position.ok_or((
    StatusCode::FORBIDDEN,
    Json(ErrorMessage {
      msg: "Click position must be provided".to_string(),
    }),
  ))?;

  let game = minesweeper::get_game_by_id(&state.pool, id)
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

  if game.belongs_to != user.id {
    return Err((
      StatusCode::UNAUTHORIZED,
      Json(ErrorMessage {
        msg: "Don't have access to this game".to_string(),
      }),
    ));
  }

  if game.state == CONSTANTS.done {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorMessage {
        msg: "Game has finished".to_string(),
      }),
    ));
  }

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

  let mut clicks = match clicks {
    Some(clicks) => {
      let click_positions: Vec<i64> = clicks.iter().map(|click| click.position).collect();

      if click_positions.contains(&click_position) {
        return Err((
          StatusCode::NOT_FOUND,
          Json(ErrorMessage {
            msg: "Square has already been clicked".to_string(),
          }),
        ));
      }

      clicks
    }
    None => Vec::new(),
  };

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

  let chance_of_winning = get_percentage_of_wining(
    CONSTANTS.game_size as f64,
    (clicks.len() as i64) + 1,
    bombs.len() as f64,
  );
  let pool = (1.0 / chance_of_winning) * ((game.stake.0 as f64) / 100.0);
  let earned = ((pool - ((game.pool.0 as f64) / 100.0)) * 100.0) as i64;

  let mut generator = Generator::new(0);

  let click = minesweeper::create_click_for_game(
    &state.pool,
    &Click {
      id: generator.generate(),
      belongs_to: game.id,
      position: click_position,
      earned: PgMoney(earned),
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
  })?[0];

  clicks.push(click);

  let response_game: minesweeper::Minesweeper = if bombs.contains(&click_position) {
    minesweeper::update_game(
      &state.pool,
      game.id,
      &UpdateGame {
        state: Some(CONSTANTS.done.to_string()),
        result: Some(CONSTANTS.lost.to_string()),
        pool: None,
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
  } else if clicks.len() as i64 == CONSTANTS.game_size - (bombs.len() as i64) {
    minesweeper::update_game(
      &state.pool,
      game.id,
      &UpdateGame {
        state: Some(CONSTANTS.done.to_string()),
        result: Some(CONSTANTS.claimed.to_string()),
        pool: Some(PgMoney((pool * 100.0) as i64)),
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
  } else {
    minesweeper::update_game(
      &state.pool,
      game.id,
      &UpdateGame {
        state: None,
        result: None,
        pool: Some(PgMoney((pool * 100.0) as i64)),
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
  };

  let response_bombs: MinesweeperBombs = if response_game.state == CONSTANTS.done {
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

  Ok((
    StatusCode::OK,
    MinesweeperGame {
      id: response_game.id.to_string(),
      state: response_game.state,
      result: response_game.result,
      stake: response_game.stake.0,
      pool: response_game.pool.0,
      bomb: response_bombs,
      size: CONSTANTS.game_size,
      clicks: clicks
        .iter()
        .map(|click| MinesweeperClick {
          earned: click.earned.0,
          position: click.position,
        })
        .collect(),
    },
  ))
}

fn get_percentage_of_wining(size: f64, next_click_count: i64, bomb_count: f64) -> f64 {
  let mut total: f64 = 1.0;

  for index in 0..next_click_count {
    total *= (size - bomb_count - (index as f64)) / (size - (index as f64));
  }
  total
}

#[derive(Serialize, Deserialize)]
pub struct ClaimGame {
  pub gameid: String,
}

pub async fn claim(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
  Json(input): Json<ClaimGame>,
) -> Result<(StatusCode, MinesweeperGame), (StatusCode, Json<ErrorMessage>)> {
  let gameid = input.gameid.parse::<i64>().unwrap();

  let game = minesweeper::get_game_by_id(&state.pool, gameid)
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
        msg: "game does not exist".to_string(),
      }),
    ))?;

  if game.belongs_to != user.id {
    return Err((
      StatusCode::UNAUTHORIZED,
      Json(ErrorMessage {
        msg: "Unathorised access to game".to_string(),
      }),
    ));
  }

  if game.state == CONSTANTS.done.to_string() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorMessage {
        msg: "Game has already been claimed or lost".to_string(),
      }),
    ));
  }

  let clicks = minesweeper::get_clicks_by_game(&state.pool, &game)
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
      StatusCode::BAD_REQUEST,
      Json(ErrorMessage {
        msg: "Must click at least once to claim".to_string(),
      }),
    ))?;

  if clicks.len() < 1 {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorMessage {
        msg: "Must click at least once to claim".to_string(),
      }),
    ));
  }

  let response_game = minesweeper::update_game(
    &state.pool,
    gameid,
    &UpdateGame {
      state: Some(CONSTANTS.done.to_string()),
      result: Some(CONSTANTS.claimed.to_string()),
      pool: None,
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

  user::update_balance(&state.pool, user.id, Some(game.pool + user.balance))
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
    MinesweeperGame {
      id: response_game.id.to_string(),
      state: response_game.state,
      result: response_game.result,
      stake: response_game.stake.0,
      pool: response_game.pool.0,
      bomb: MinesweeperBombs {
        count: bombs.len() as i64,
        position: bombs,
      },
      size: CONSTANTS.game_size,
      clicks: clicks
        .iter()
        .map(|click| MinesweeperClick {
          earned: click.earned.0,
          position: click.position,
        })
        .collect(),
    },
  ))
}
