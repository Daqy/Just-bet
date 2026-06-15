use std::sync::Arc;

use crate::AppState;
use crate::controllers::games::minesweeper::CONSTANTS;
use crate::models::user::{self, UpdateUser};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};

use diesel::data_types::PgMoney;

use argon2::{
  Argon2,
  password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, decode_header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use snowflaked::Generator;
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
  pub msg: String,
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
  let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();
  if Argon2::default()
    .verify_password(&input.password.as_bytes(), &parsed_hash)
    .is_err()
  {
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
      exp: (Utc::now().naive_utc() + Duration::seconds(10))
        .and_utc()
        .timestamp() as usize,
    },
    &EncodingKey::from_secret("secret".as_ref()),
  )
  .unwrap();

  let cookie = Cookie::build(("token", token))
    .path("/api")
    .secure(true)
    .http_only(true)
    .same_site(SameSite::Strict)
    .max_age(time::Duration::seconds(Duration::days(1).num_seconds()))
    .build();

  Ok((StatusCode::OK, cookies.add(cookie)))
}

pub async fn logout(cookies: CookieJar) -> CookieJar {
  cookies.remove("token")
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
    })?;

  match user_exist {
    Some(_) => {
      return Err((
        StatusCode::CONFLICT,
        Json(ErrorMessage {
          msg: "Username already exist".to_string(),
        }),
      ));
    }
    None => {}
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
    })?;

  match email_exist {
    Some(_) => {
      return Err((
        StatusCode::CONFLICT,
        Json(ErrorMessage {
          msg: "Email already exist".to_string(),
        }),
      ));
    }
    None => {}
  }

  let password = input.password.as_bytes();
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();

  let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();

  let mut generator = Generator::new(0);

  let user = user::create_user(
    &state.pool,
    &user::User {
      id: generator.generate(),
      username: input.username,
      email: input.email,
      password_hash: password_hash,
      balance: PgMoney(10000), // £100
      claim_expires_timestamp: Utc::now().naive_utc(),
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

  let cookie = Cookie::build(("token", token))
    .path("/")
    .secure(true)
    .http_only(true)
    // .max_age(Duration::days(1))
    .build();

  Ok((StatusCode::OK, cookies.add(cookie)))
}

pub async fn verify(
  State(state): State<Arc<AppState>>,
  cookies: CookieJar,
  mut request: Request,
  next: Next,
  // Json(input): Json<RegisterUser>,
) -> Result<Response, (StatusCode, Json<ErrorMessage>)> {
  let token = cookies
    .get("token")
    .ok_or((
      StatusCode::UNAUTHORIZED,
      Json(ErrorMessage {
        msg: "A token is required for authentication".to_string(),
      }),
    ))?
    .value();

  let user_id = decode_header(token)
    .map_err(|_| {
      (
        StatusCode::FORBIDDEN,
        Json(ErrorMessage {
          msg: "Token is no longer valid".to_string(),
        }),
      )
    })?
    .kid
    .ok_or((
      StatusCode::UNAUTHORIZED,
      Json(ErrorMessage {
        msg: "A token is required for authentication".to_string(),
      }),
    ))?
    .parse::<i64>()
    .unwrap();

  let user = user::user_exist_by_id(&state.pool, &user_id)
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Database error".to_string(),
        }),
      )
    })?
    .ok_or((
      StatusCode::UNAUTHORIZED,
      Json(ErrorMessage {
        msg: "A valid token is required for authentication".to_string(),
      }),
    ))?;

  request.extensions_mut().insert(user);
  Ok(next.run(request).await)
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
  username: String,
  balance: i64,
}

#[axum::debug_handler]
pub async fn get(
  State(_state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorMessage>)> {
  Ok((
    StatusCode::OK,
    Json(UserResponse {
      username: user.username,
      balance: user.balance.0,
    }),
  ))
}

#[axum::debug_handler]
pub async fn auth(
  cookies: CookieJar,
) -> Result<(StatusCode, CookieJar), (StatusCode, Json<ErrorMessage>)> {
  Ok((StatusCode::OK, cookies))
}

#[derive(Serialize, Deserialize)]
pub struct Balance {
  balance: i64,
}
impl IntoResponse for Balance {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

#[axum::debug_handler]
pub async fn get_balance(
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, Balance), (StatusCode, Json<ErrorMessage>)> {
  Ok((
    StatusCode::OK,
    Balance {
      balance: user.balance.0,
    },
  ))
}

#[derive(Serialize)]
pub struct GetClaim {
  claimable: bool,
  claim_amount: i64,
  timestamp: chrono::NaiveDateTime,
}
impl IntoResponse for GetClaim {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

#[axum::debug_handler]
pub async fn get_claim(
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, GetClaim), (StatusCode, Json<ErrorMessage>)> {
  if user.claim_expires_timestamp < Utc::now().naive_utc() {
    return Ok((
      StatusCode::OK,
      GetClaim {
        claimable: true,
        claim_amount: CONSTANTS.claim_amount.0,
        timestamp: user.claim_expires_timestamp,
      },
    ));
  }

  return Ok((
    StatusCode::OK,
    GetClaim {
      claimable: false,
      claim_amount: 50,
      timestamp: user.claim_expires_timestamp,
    },
  ));
  // Ok((
  //   StatusCode::OK,
  //   Balance {
  //     balance: user.balance.0,
  //   },
  // ))
}

#[derive(Serialize)]
pub struct Claim {
  claimable: bool,
  claim_amount: i64,
  timestamp: chrono::NaiveDateTime,
  balance: i64,
}
impl IntoResponse for Claim {
  fn into_response(self) -> Response {
    Json(json!(&self)).into_response()
  }
}

pub async fn claim(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<user::User>,
) -> Result<(StatusCode, Claim), (StatusCode, Json<ErrorMessage>)> {
  if user.claim_expires_timestamp < Utc::now().naive_utc() {
    let reponse_user = user::update(
      &state.pool,
      user.id,
      &UpdateUser {
        timestamp: Some(Utc::now().naive_utc() + Duration::days(1)),
        balance: Some(user.balance + CONSTANTS.claim_amount),
      },
    )
    .await
    .map_err(|_| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorMessage {
          msg: "Database error".to_string(),
        }),
      )
    })?;

    return Ok((
      StatusCode::OK,
      Claim {
        claimable: false,
        claim_amount: CONSTANTS.claim_amount.0,
        timestamp: reponse_user.claim_expires_timestamp,
        balance: reponse_user.balance.0,
      },
    ));
  }

  Ok((
    StatusCode::OK,
    Claim {
      claimable: false,
      claim_amount: CONSTANTS.claim_amount.0,
      timestamp: user.claim_expires_timestamp,
      balance: user.balance.0,
    },
  ))
}
