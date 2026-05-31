use diesel::{
  BelongingToDsl, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, Selectable,
  SelectableHelper,
  associations::{Associations, Identifiable},
  data_types::PgMoney,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};

use crate::models::schema::minesweeper;

#[derive(Identifiable, Debug, Queryable, Selectable)]
#[diesel(table_name = crate::models::schema::minesweeper)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Minesweeper {
  pub id: i64,
  pub belongs_to: i64,
  pub state: String,
  pub result: String,
  pub stake: PgMoney,
  pub pool: PgMoney,
}

#[derive(Identifiable, Debug, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Minesweeper, foreign_key = id))]
#[diesel(table_name = crate::models::schema::bombs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Bomb {
  pub id: i64,
  pub belongs_to: i64,
  pub position: i64,
}

#[derive(Identifiable, Debug, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Minesweeper, foreign_key = id))]
#[diesel(table_name = crate::models::schema::clicks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Click {
  pub id: i64,
  pub belongs_to: i64,
  pub position: i64,
  pub earned: PgMoney,
}

pub async fn get_game_by_user_id(
  pool: &Pool<AsyncPgConnection>,
  id: &i64,
) -> Result<Option<Minesweeper>, diesel::result::Error> {
  Ok(
    minesweeper::table
      .filter(minesweeper::belongs_to.eq(id))
      .select(Minesweeper::as_select())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn get_clicks_by_game(
  pool: &Pool<AsyncPgConnection>,
  game: &Minesweeper,
) -> Result<Option<Vec<Click>>, diesel::result::Error> {
  Ok(
    Click::belonging_to(&game)
      .select(Click::as_select())
      .get_results(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn get_bombs_by_game(
  pool: &Pool<AsyncPgConnection>,
  game: &Minesweeper,
) -> Result<Option<Vec<Bomb>>, diesel::result::Error> {
  Ok(
    Bomb::belonging_to(&game)
      .select(Bomb::as_select())
      .get_results(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}
