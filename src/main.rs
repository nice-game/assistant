#![feature(once_cell)]

mod graphql;

use crate::graphql::{Context, Mutation, Query, Schema, Subscription};
use dotenv::dotenv;
use futures::FutureExt;
use juniper::InputValue;
use juniper_graphql_ws::ConnectionConfig;
use juniper_warp::{graphiql_filter, make_graphql_filter, subscriptions::serve_graphql_ws};
use log::{error, LevelFilter};
use simple_logger::SimpleLogger;
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use warp::{
	filters::{header, log::log},
	get, hyper, post,
	reject::Reject,
	reply::with::header,
	ws::Ws,
	Filter, Rejection,
};

#[tokio::main]
async fn main() {
	SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();

	dotenv().ok();

	let qraphql_schema = || Schema::new(Query, Mutation, Subscription);

	let graphql_http = post().and(make_graphql_filter(qraphql_schema(), graphql_context().boxed()));
	let root_node = Arc::new(qraphql_schema());
	let graphql_ws = warp::ws()
		.map(move |ws: Ws| {
			let root_node = root_node.clone();
			ws.on_upgrade(move |websocket| async move {
				// must be shorter than 10 seconds to prevent graphiql from killing the connection
				let keep_alive_interval = Duration::from_secs(8);
				let connection_config = move |payload: HashMap<String, InputValue>| async move {
					let session_uuid = payload.get("sessionUuid").and_then(|x| x.as_string_value());
					let ctx = Context::new(session_uuid.map(|session_uuid| session_uuid.parse().unwrap()));
					Ok(ConnectionConfig::new(ctx).with_keep_alive_interval(keep_alive_interval))
						as Result<_, Infallible>
				};
				serve_graphql_ws(websocket, root_node.clone(), connection_config)
					.map(|r| {
						if let Err(e) = r {
							error!("Websocket error: {}", e);
						}
					})
					.await
			})
		})
		.with(header("Sec-WebSocket-Protocol", "graphql-ws"));
	let graphql = warp::path("graphql").and(graphql_http.or(graphql_ws));
	let graphiql = warp::path("graphiql").and(get()).and(graphiql_filter("/graphql", Some("/graphql")));

	let routes = graphql.or(graphiql).with(log(""));
	warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

#[derive(Debug)]
struct HyperReject(hyper::Error);
impl Reject for HyperReject {}
impl From<hyper::Error> for HyperReject {
	fn from(err: hyper::Error) -> Self {
		Self(err)
	}
}

fn graphql_context() -> impl Filter<Extract = (Context,), Error = Rejection> + Clone + Send + Sync + 'static {
	header::optional("Authorization").map(|session_uuid: Option<String>| {
		Context::new(session_uuid.map(|session_uuid| session_uuid.parse().unwrap()))
	})
}
