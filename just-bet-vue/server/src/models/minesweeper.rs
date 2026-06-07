use diesel::{
  BelongingToDsl, ExpressionMethods, OptionalExtension, QueryDsl, Queryable, Selectable,
  SelectableHelper,
  associations::{Associations, Identifiable},
  data_types::PgMoney,
  prelude::Insertable,
  query_builder::AsChangeset,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};

use crate::models::schema::{
  bombs::{self},
  clicks, minesweeper,
};

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
#[diesel(belongs_to(Minesweeper, foreign_key = belongs_to))]
#[diesel(table_name = crate::models::schema::bombs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Bomb {
  pub id: i64,
  pub belongs_to: i64,
  pub position: i64,
}

#[derive(Identifiable, Debug, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Minesweeper, foreign_key = belongs_to))]
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
      .order_by(minesweeper::created.desc())
      .first(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

pub async fn get_game_by_id(
  pool: &Pool<AsyncPgConnection>,
  id: i64,
) -> Result<Option<Minesweeper>, diesel::result::Error> {
  Ok(
    minesweeper::table
      .filter(minesweeper::id.eq(id))
      .select(Minesweeper::as_select())
      .order_by(minesweeper::created.desc())
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
    Bomb::belonging_to(game)
      .select(Bomb::as_select())
      .get_results(&mut pool.get().await.unwrap())
      .await
      .optional()?,
  )
}

#[derive(Insertable)]
#[diesel(table_name = minesweeper)]
struct NewGame<'a> {
  id: &'a i64,
  belongs_to: &'a i64,
  state: &'a str,
  result: &'a str,
  stake: &'a PgMoney,
  pool: &'a PgMoney,
}

pub async fn create_game(
  pool: &Pool<AsyncPgConnection>,
  game: &Minesweeper,
) -> Result<Vec<Minesweeper>, diesel::result::Error> {
  Ok(
    diesel::insert_into(minesweeper::table)
      .values(&NewGame {
        id: &game.id,
        belongs_to: &game.belongs_to,
        state: game.state.as_str(),
        result: game.result.as_str(),
        stake: &game.stake,
        pool: &game.pool,
      })
      .returning(Minesweeper::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?,
  )
}

#[derive(Insertable, Clone, Copy)]
#[diesel(table_name = bombs)]
pub struct NewBomb {
  pub id: i64,
  pub belongs_to: i64,
  pub position: i64,
}

pub async fn create_bomb_for_game<'a>(
  pool: &Pool<AsyncPgConnection>,
  bombs: &Vec<NewBomb>,
) -> Result<Vec<Bomb>, diesel::result::Error> {
  Ok(
    diesel::insert_into(bombs::table)
      .values(bombs)
      .returning(Bomb::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?,
  )
}

#[derive(Insertable)]
#[diesel(table_name = clicks)]
struct NewClick<'a> {
  id: &'a i64,
  belongs_to: &'a i64,
  position: &'a i64,
  earned: &'a PgMoney,
}

pub async fn create_click_for_game(
  pool: &Pool<AsyncPgConnection>,
  click: &Click,
) -> Result<Vec<Click>, diesel::result::Error> {
  Ok(
    diesel::insert_into(clicks::table)
      .values(&NewClick {
        id: &click.id,
        belongs_to: &click.belongs_to,
        position: &click.position,
        earned: &click.earned,
      })
      .returning(Click::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?,
  )
}

#[derive(AsChangeset)]
#[diesel(table_name = minesweeper)]
pub struct SetGame<'a> {
  pub state: Option<&'a String>,
  pub result: Option<&'a String>,
  pub pool: Option<PgMoney>,
}

pub struct UpdateGame {
  pub state: Option<String>,
  pub result: Option<String>,
  pub pool: Option<PgMoney>,
}

pub async fn update_game(
  pool: &Pool<AsyncPgConnection>,
  game_id: i64,
  game: &UpdateGame,
) -> Result<Minesweeper, diesel::result::Error> {
  Ok(
    diesel::update(minesweeper::table)
      .filter(minesweeper::id.eq(game_id))
      .set(&SetGame {
        state: game.state.as_ref(),
        result: game.result.as_ref(),
        pool: game.pool,
      })
      .returning(Minesweeper::as_returning())
      .get_results(&mut pool.get().await.unwrap())
      .await?
      .pop()
      .unwrap(),
  )
}
