use std::sync::Arc;

use axum::{
  Extension, Json,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
  AppState,
  controllers::user::ErrorMessage,
  models::{minesweeper, user},
};

type MinesweeperBombs = Vec<i64>;

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
  pub bombs: MinesweeperBombs,
  pub clicks: Vec<MinesweeperClick>,
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

  let bombs: MinesweeperBombs = minesweeper::get_bombs_by_game(&state.pool, &game)
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
        bombs: bombs,
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
        stake: game.stake.0,
        pool: game.pool.0,
        bombs: bombs,
        clicks: Vec::new(),
      },
    )),
  }
}
