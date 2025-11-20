use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::models::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct User {
  pub id: i64,
  pub username: String,
  pub email: String,
  pub password_hash: String,
  pub balance: i64,
  pub claim_expires_timestamp: i32,
}