use db::{model::Session, DB};
use diesel::prelude::*;
use futures::{future::ok, Future, FutureExt};
use std::{
	pin::Pin,
	sync::{Arc, Mutex},
};
use tokio::{
	spawn,
	task::{spawn_blocking, JoinError},
};
use uuid::Uuid;

pub struct Context {
	session: Arc<Mutex<LazySession>>,
}
impl Context {
	pub fn new(session_uuid: Option<Uuid>) -> Self {
		let session = match session_uuid {
			Some(session_uuid) => LazySession::Uuid(session_uuid),
			None => LazySession::None,
		};
		Self { session: Arc::new(Mutex::new(session)) }
	}

	// this is ugly as fuck but it allows us to drop Context and keep the future that load_session returns
	// TODO: beautify this shit
	pub fn load_session(&self) -> Pin<Box<dyn Future<Output = Result<Option<Session>, JoinError>> + Send>> {
		match &*self.session.lock().unwrap() {
			LazySession::None => ok(None).boxed() as _,
			&LazySession::Uuid(session_uuid) => {
				let self_session = self.session.clone();
				spawn(async move {
					let session = spawn_blocking(move || {
						use db::schema::sessions::dsl::*;
						sessions
							.filter(uuid.eq(session_uuid).and(expired.eq(false)))
							.first::<Session>(&DB.get().unwrap())
							.unwrap()
					})
					.await
					.unwrap();
					*self_session.lock().unwrap() = LazySession::Session(session.clone());
					Some(session)
				})
				.boxed()
			},
			LazySession::Session(session) => ok(Some(session.clone())).boxed(),
		}
	}
}
impl juniper::Context for Context {}

enum LazySession {
	None,
	Uuid(Uuid),
	Session(Session),
}
