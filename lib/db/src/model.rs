use crate::schema::{ai_requests, sessions, users};
use chrono::{DateTime, Utc};
use diesel::Queryable;
use uuid::Uuid;

#[derive(Clone, Debug, Queryable)]
pub struct AiRequest {
	pub id: i32,
	pub session_id: i32,
	pub query: String,
	pub query_created: DateTime<Utc>,
	pub reply: Option<String>,
	pub reply_created: Option<DateTime<Utc>>,
}
impl AiRequest {
	pub fn from_new(id: i32, new: NewAiRequest) -> Self {
		Self {
			id,
			session_id: new.session_id,
			query: new.query.to_owned(),
			query_created: new.query_created,
			reply: new.reply.map(|x| x.to_owned()),
			reply_created: new.reply_created,
		}
	}
}

#[derive(Insertable)]
#[table_name = "ai_requests"]
pub struct NewAiRequest<'a> {
	pub session_id: i32,
	pub query: &'a str,
	pub query_created: DateTime<Utc>,
	pub reply: Option<&'a str>,
	pub reply_created: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Queryable)]
pub struct User {
	pub id: i32,
	pub username: String,
	pub password: Vec<u8>,
	pub salt: Vec<u8>,
}
impl User {
	pub fn from_new(id: i32, new: NewUser) -> Self {
		Self { id, username: new.username, password: new.password.to_owned(), salt: new.salt.to_owned() }
	}
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	pub username: String,
	pub password: &'a [u8],
	pub salt: &'a [u8],
}

#[derive(Clone, Debug, Queryable)]
pub struct Session {
	pub id: i32,
	pub uuid: Uuid,
	pub user_id: Option<i32>,
	pub expired: bool,
}
impl Session {
	pub fn from_new(id: i32, new: NewSession) -> Self {
		Self { id, uuid: new.uuid, user_id: new.user_id, expired: new.expired }
	}
}

#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
	pub uuid: Uuid,
	pub user_id: Option<i32>,
	pub expired: bool,
}
