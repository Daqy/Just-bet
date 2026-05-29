use diesel::{
  ExpressionMethods, OptionalExtension, QueryDsl, Queryable, Selectable, SelectableHelper,
  data_types::{PgMoney, PgTimestamp},
  prelude::Insertable,
};
// use diesel::prelude::*;

use crate::models::schema::users;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::models::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
#[derive(Clone)]
pub struct User {
  pub id: i64,
  pub username: String,
  pub email: String,
  pub password_hash: String,
  pub balance: PgMoney,
  pub claim_expires_timestamp: PgTimestamp,
}

pub async fn user_exist_by_username(
  pool: &Pool<AsyncPgConnection>,
  username: &str,
) -> Result<Option<User>, diesel::result::Error> {
  Ok(
    users::table
      .filter(users::username.eq(username))
      .select(User::as_select())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn user_exist_by_email(
  pool: &Pool<AsyncPgConnection>,
  email: &str,
) -> Result<Option<User>, diesel::result::Error> {
  Ok(
    users::table
      .filter(users::email.eq(email))
      .select(User::as_select())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn user_exist_by_id(
  pool: &Pool<AsyncPgConnection>,
  id: i64,
) -> Result<Option<User>, diesel::result::Error> {
  Ok(
    users::table
      .filter(users::id.eq(id))
      .select(User::as_select())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
  id: &'a i64,
  username: &'a str,
  email: &'a str,
  password_hash: &'a str,
  balance: &'a PgMoney,
  claim_expires_timestamp: &'a PgTimestamp,
}
pub async fn create_user(
  pool: &Pool<AsyncPgConnection>,
  user: User,
) -> Result<Vec<User>, diesel::result::Error> {
  Ok(
    diesel::insert_into(users::table)
      .values(&NewUser {
        id: &user.id,
        username: user.username.as_str(),
        email: user.email.as_str(),
        password_hash: user.password_hash.as_str(),
        balance: &user.balance,
        claim_expires_timestamp: &user.claim_expires_timestamp,
      })
      .returning(User::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?,
  )
}
