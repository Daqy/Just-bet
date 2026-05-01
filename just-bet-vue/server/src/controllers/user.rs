use std::sync::Arc;

// use axum::extract::State;
use crate::AppState;
use crate::models::user;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;

use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Serialize, Deserialize)]
pub struct LoginUser {
  username: String,
  password: String,
}
impl IntoResponse for LoginUser {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
  msg: String,
}
impl IntoResponse for ErrorMessage {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  exp: usize,
}

#[axum::debug_handler]
pub async fn login(
  State(state): State<Arc<AppState>>,
  cookies: CookieJar,
  Json(input): Json<LoginUser>,
) -> Result<(StatusCode, CookieJar), (StatusCode, Json<ErrorMessage>)> {
  let user = user::user_exist_by_username(&state.pool, &input.username)
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
        msg: "Either username or password is incorrect".to_string(),
      }),
    ))?;

  // todo: Pasword needs to be hashed (however, current saved one isn't hashed)
  if user.password_hash != input.password {
    return Err((
      StatusCode::OK,
      Json(ErrorMessage {
        msg: "Either username or password is incorrect".to_string(),
      }),
    ));
  };

  let token = encode(
    &Header {
      kid: Some(user.id.to_string()),
      ..Default::default()
    },
    &Claims {
      exp: Utc::now().timestamp() as usize,
    },
    &EncodingKey::from_secret("secret".as_ref()),
  )
  .unwrap();

  let cookie = Cookie::build(("TOKEN", token))
    .path("/")
    .secure(true)
    .http_only(true)
    // .max_age(Duration::days(1))
    .build();

  Ok((StatusCode::OK, cookies.add(cookie)))
}

#[derive(Serialize, Deserialize)]
pub struct RegisterUser {
  email: String,
  username: String,
  password: String,
}
impl IntoResponse for RegisterUser {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

pub async fn register(
  State(state): State<Arc<AppState>>,
  cookies: CookieJar,
  Json(input): Json<RegisterUser>,
) -> Result<(StatusCode, CookieJar), (StatusCode, Json<ErrorMessage>)> {
  let user_exist = user::user_exist_by_username(&state.pool, &input.username)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?
    .unwrap();

  if user_exist.username_exist() {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorMessage {
        msg: "Username already exist".to_string(),
      }),
    ));
  }

  let email_exist = user::user_exist_by_email(&state.pool, &input.email)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Something really went wrong".to_string(),
        }),
      )
    })?
    .unwrap();

  if email_exist.email_exist() {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorMessage {
        msg: "Email already exist".to_string(),
      }),
    ));
  }

  let password = input.password;

  let user = user::create_user(
    &state.pool,
    user::User {
      id: 1,
      username: input.username,
      email: input.email,
      password_hash: password,
      balance: diesel::data_types::PgMoney(100),
      claim_expires_timestamp: diesel::data_types::PgTimestamp(Utc::now().timestamp()),
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

  let token = encode(
    &Header {
      kid: Some(user.id.to_string()),
      ..Default::default()
    },
    &Claims {
      exp: Utc::now().timestamp() as usize,
    },
    &EncodingKey::from_secret("secret".as_ref()),
  )
  .unwrap();

  let cookie = Cookie::build(("TOKEN", token))
    .path("/")
    .secure(true)
    .http_only(true)
    // .max_age(Duration::days(1))
    .build();

  Ok((StatusCode::OK, cookies.add(cookie)))
}
