use diesel::data_types::{PgMoney, PgTimestamp};
use diesel::prelude::*;

use crate::models::schema::users;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::models::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct User {
  pub id: i64,
  pub username: String,
  pub email: String,
  pub password_hash: String,
  pub balance: PgMoney,
  pub claim_expires_timestamp: PgTimestamp,
}

pub async fn user_exist_by_username(conn: &mut PgConnection, username: &str) -> Result<Option<User>, diesel::result::Error> {
  Ok(users::table.filter(users::username.eq(username)).select(User::as_select()).first(conn).optional()?)
}