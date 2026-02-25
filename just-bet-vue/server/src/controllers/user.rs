use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use crate::establish_connection;
use crate::models::user;

struct LoginUser {
  username: String,
  password: String,
}
pub async fn login(
  Json(input): Json<LoginUser>,
) -> Result<Json<LoginUser>, StatusCode> {
  // let connection = &mut establish_connection();

  // let user = user::user_exist_by_username(pool, &input.username)
  //   .await
  //   .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
  //   .ok_or(StatusCode::UNAUTHORIZED)?;
  //
  Ok(Json(input))
}