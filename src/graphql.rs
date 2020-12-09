// required to make graphql_schema_from_file stop generating warnings
#![allow(unused_braces)]

mod context;

pub use context::Context;
use db::model::{NewUser, User};
use tokio::task::JoinError;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crypto::bcrypt::bcrypt;
use db::{
	model::{AiRequest as DbAiRequest, NewAiRequest, NewSession, Session as DbSession},
	DB,
};
use diesel::prelude::*;
use futures::{channel::oneshot, lock::Mutex, FutureExt, Stream, StreamExt};
use juniper::{Executor, FieldResult};
use juniper_from_schema::graphql_schema_from_file;
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use std::{collections::HashMap, pin::Pin};
use tokio::{spawn, task::spawn_blocking};
use uuid::Uuid;

lazy_static! {
	/// ai_request_id -> AiRequest
	static ref REPLY_STREAMS: Mutex<HashMap<i32, oneshot::Sender<AiRequest>>> = Mutex::default();
}

graphql_schema_from_file!("schema.graphql");

pub struct Query;
#[async_trait]
impl QueryFields for Query {
	async fn field_ai_request<'s, 'r, 'a>(
		&'s self,
		_: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, AiRequest, Walked>,
		ai_request_id: i32,
	) -> FieldResult<AiRequest> {
		Ok(spawn_blocking(move || {
			use db::schema::ai_requests::dsl::*;

			AiRequest(ai_requests.find(ai_request_id).first(&DB.get().unwrap()).unwrap())
		})
		.await?)
	}

	async fn field_unanswered_ai_requests<'s, 'r, 'a>(
		&'s self,
		_: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, AiRequest, Walked>,
	) -> FieldResult<Vec<AiRequest>> {
		Ok(spawn_blocking(move || {
			use db::schema::ai_requests::dsl::*;

			ai_requests
				.filter(reply.is_null())
				.order(query_created.asc())
				.load(&DB.get().unwrap())
				.unwrap()
				.into_iter()
				.map(|x| AiRequest(x))
				.collect()
		})
		.await?)
	}
}

pub struct Mutation;
#[async_trait]
impl MutationFields for Mutation {
	async fn field_register<'s, 'r, 'a>(
		&'s self,
		executor: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, Session, Walked>,
		username: String,
		password: String,
	) -> FieldResult<Session> {
		let user_id = spawn_blocking(move || {
			use db::schema::users;

			let conn = DB.get().unwrap();

			let salt: [u8; 16] = thread_rng().gen();
			let mut password_hash = [0; 24];
			bcrypt(12, &salt, password.as_bytes(), &mut password_hash);

			let new_user = NewUser { username, password: &password_hash, salt: &salt };
			diesel::insert_into(users::table).values(&new_user).returning(users::id).get_result(&conn).unwrap()
		});
		let user_id = user_id.await?;

		let session = executor.context().load_session().await?;
		Ok(Session(get_or_create_session(session, user_id).await?))
	}

	async fn field_auth_anon<'s, 'r, 'a>(
		&'s self,
		_: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, Session, Walked>,
	) -> FieldResult<Session> {
		Ok(Session(create_session(None).await?))
	}

	async fn field_auth_user<'s, 'r, 'a>(
		&'s self,
		executor: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, Session, Walked>,
		username: String,
		password: String,
	) -> FieldResult<Session> {
		let user_id = spawn_blocking(move || {
			use db::schema::users;

			let conn = DB.get().unwrap();

			log::debug!("login username {}", username);
			let user: User = users::table.filter(users::username.eq(username)).first(&conn).unwrap();

			let mut password_hash = [0; 24];
			bcrypt(12, &user.salt, password.as_bytes(), &mut password_hash);
			if &password_hash != &*user.password {
				return Err("Invalid username or password");
			}

			Ok(user.id)
		});
		let user_id = user_id.await??;

		let session = executor.context().load_session().await?;
		Ok(Session(get_or_create_session(session, user_id).await?))
	}

	async fn field_logout<'s, 'r, 'a>(&'s self, executor: &Executor<'r, 'a, Context>) -> FieldResult<bool> {
		let session = match executor.context().load_session().await? {
			Some(session) => session,
			None => return Ok(false),
		};

		spawn_blocking(move || {
			use db::schema::sessions;

			diesel::update(sessions::table)
				.filter(sessions::uuid.eq(session.uuid))
				.set(sessions::expired.eq(true))
				.execute(&DB.get().unwrap())
		})
		.await??;

		Ok(true)
	}

	async fn field_create_ai_request<'s, 'r, 'a>(
		&'s self,
		executor: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, AiRequest, Walked>,
		text: String,
	) -> FieldResult<AiRequest> {
		let sessionid = executor.context().load_session().await?.unwrap().id;
		Ok(spawn_blocking(move || {
			use db::schema::ai_requests::dsl::*;

			let conn = DB.get().unwrap();

			let new_ai_request = NewAiRequest {
				session_id: sessionid,
				query: &text,
				query_created: Utc::now(),
				reply: None,
				reply_created: None,
			};
			let ai_request_id =
				diesel::insert_into(ai_requests).values(&new_ai_request).returning(id).get_result(&conn).unwrap();

			AiRequest(DbAiRequest::from_new(ai_request_id, new_ai_request))
		})
		.await?)
	}

	async fn field_create_ai_reply<'s, 'r, 'a>(
		&'s self,
		_: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, AiRequest, Walked>,
		ai_request_id: i32,
		text: String,
	) -> FieldResult<AiRequest> {
		let ai_request: DbAiRequest = spawn_blocking(move || {
			use db::schema::ai_requests::dsl::*;

			let conn = DB.get().unwrap();

			diesel::update(ai_requests.find(ai_request_id))
				.set((reply.eq(text), reply_created.eq(Utc::now().naive_utc())))
				.get_result(&conn)
				.unwrap()
		})
		.await?;

		log::debug!("before remove {:?}", *REPLY_STREAMS.lock().await);
		if let Some(reply_stream) = REPLY_STREAMS.lock().await.remove(&ai_request_id) {
			reply_stream.send(AiRequest(ai_request.clone())).unwrap();
		}

		Ok(AiRequest(ai_request))
	}
}

pub struct Subscription;
#[async_trait]
impl SubscriptionFields for Subscription {
	async fn field_ai_reply<'s, 'r, 'a>(
		&'s self,
		executor: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, AiRequest, Walked>,
		ai_request_id: i32,
	) -> FieldResult<Pin<Box<(dyn Stream<Item = AiRequest> + Send + 'static)>>> {
		log::debug!("in subscription");

		let (send, recv) = oneshot::channel();

		let mut reply_streams = REPLY_STREAMS.lock().await;
		reply_streams.insert(ai_request_id, send);

		let session = executor.context().load_session();
		spawn(async move {
			let session = session.await.unwrap().unwrap();
			let ai_request = spawn_blocking(move || {
				use db::schema::ai_requests::dsl::*;

				let conn = DB.get().unwrap();
				ai_requests.filter(session_id.eq(session.id)).find(ai_request_id).first::<DbAiRequest>(&conn).unwrap()
			})
			.await
			.unwrap();

			if ai_request.reply.is_some() {
				REPLY_STREAMS.lock().await.remove(&ai_request_id).unwrap().send(AiRequest(ai_request)).unwrap();
			}
		});

		Ok(Box::pin(recv.into_stream().map(|x| x.unwrap())))
	}
}

#[derive(Debug)]
pub struct AiRequest(DbAiRequest);
#[async_trait]
impl AiRequestFields for AiRequest {
	fn field_id<'s, 'r, 'a>(&'s self, _: &Executor<'r, 'a, Context>) -> FieldResult<&'s i32> {
		Ok(&self.0.id)
	}

	fn field_session_id<'s, 'r, 'a>(&'s self, _: &Executor<'r, 'a, Context>) -> FieldResult<&'s i32> {
		Ok(&self.0.session_id)
	}

	fn field_query<'s, 'r, 'a>(&'s self, _: &Executor<'r, 'a, Context>) -> FieldResult<&'s String> {
		Ok(&self.0.query)
	}

	fn field_query_created<'s, 'r, 'a>(&'s self, _: &Executor<'r, 'a, Context>) -> FieldResult<&'s DateTime<Utc>> {
		Ok(&self.0.query_created)
	}

	fn field_reply<'s, 'r, 'a>(&'s self, _: &Executor<'r, 'a, Context>) -> FieldResult<&'s Option<String>> {
		Ok(&self.0.reply)
	}

	fn field_reply_created<'s, 'r, 'a>(
		&'s self,
		_: &Executor<'r, 'a, Context>,
	) -> FieldResult<&'s Option<DateTime<Utc>>> {
		Ok(&self.0.reply_created)
	}

	async fn field_history<'s, 'r, 'a>(
		&'s self,
		_: &Executor<'r, 'a, Context>,
		_: &QueryTrail<'r, AiRequest, Walked>,
	) -> FieldResult<Vec<AiRequest>> {
		let self_query_created = self.0.query_created;
		let self_session_id = self.0.session_id;
		Ok(spawn_blocking(move || {
			use db::schema::ai_requests::dsl::*;

			ai_requests
				.filter(query_created.lt(self_query_created).and(session_id.eq(self_session_id)))
				.order(query_created.asc())
				.load(&DB.get().unwrap())
				.unwrap()
				.into_iter()
				.map(|x| AiRequest(x))
				.collect()
		})
		.await?)
	}
}

pub struct Session(DbSession);
impl SessionFields for Session {
	fn field_uuid<'s, 'r, 'a>(&'s self, _: &Executor<'r, 'a, Context>) -> FieldResult<&'s Uuid> {
		Ok(&self.0.uuid)
	}
}

async fn get_or_create_session(session: Option<DbSession>, user_id: i32) -> Result<DbSession, JoinError> {
	if let Some(session) = session {
		spawn_blocking(move || {
			use db::schema::sessions;

			let conn = DB.get().unwrap();
			diesel::update(sessions::table)
				.filter(sessions::uuid.eq(session.uuid))
				.set(sessions::user_id.eq(user_id))
				.execute(&conn)
				.unwrap();
			session
		})
		.await
	} else {
		create_session(Some(user_id)).await
	}
}

async fn create_session(user_id: Option<i32>) -> Result<DbSession, JoinError> {
	spawn_blocking(move || {
		use db::schema::sessions;

		let new_session = NewSession { uuid: Uuid::new_v4(), user_id, expired: false };
		let session_id = diesel::insert_into(sessions::table)
			.values(&new_session)
			.returning(sessions::id)
			.get_result(&DB.get().unwrap())
			.unwrap();
		DbSession::from_new(session_id, new_session)
	})
	.await
}
