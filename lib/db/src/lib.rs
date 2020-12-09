#[macro_use]
extern crate diesel;

pub mod model;
pub mod schema;

use diesel::{
	prelude::*,
	r2d2::{ConnectionManager, Pool, PooledConnection},
};
use lazy_static::lazy_static;
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

lazy_static! {
	pub static ref DB: PgPool = {
		let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
		let manager = ConnectionManager::<PgConnection>::new(&database_url);
		Pool::new(manager).expect("Connection pool could not be created")
	};
}
