use std::{env, time::Duration};

use deadpool_diesel::{
	Runtime,
	postgres::{Manager, Pool},
};
use diesel::prelude::*;
use dotenvy::dotenv;
use tokio::sync::OnceCell;

use crate::*;

static POSTGRES_POOL: OnceCell<Pool> = OnceCell::const_new();

/// The default (async) test runner for Postgres. It uses `deadpool-diesel` to create a connection pool that can be shared among tests, so that they can be executed faster.
///
/// It requires setting the env `DATABASE_URL` (via regular env or `.env` file) to set up the connection pool.
///
/// If you want the [`PgEnum`] macro invocations to use this as the `async_runner` by default, you can use the `default-pg-runner` feature.
pub struct AsyncPgRunner;

impl AsyncTestRunner<PgConnection> for AsyncPgRunner {
	async fn run_check<F>(f: F) -> Result<(), DbEnumError>
	where
		F: FnOnce(&mut PgConnection) -> Result<(), DbEnumError> + Send + 'static,
	{
		POSTGRES_POOL
			.get_or_init(|| async { create_pg_pool() })
			.await
			.get()
			.await
			.expect("Failed to get a connection to the Postgres database")
			.interact(f)
			.await
			.expect("Postgres testing pool thread crashed")
	}
}

#[track_caller]
fn create_pg_pool() -> Pool {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL")
		.expect("Failed to set up testing pool for Postgres: DATABASE_URL is not set");

	let manager = Manager::new(database_url, Runtime::Tokio1);

	Pool::builder(manager)
		.max_size(16)
		.runtime(Runtime::Tokio1)
		.wait_timeout(Some(Duration::from_secs(5)))
		.create_timeout(Some(Duration::from_secs(5)))
		.recycle_timeout(Some(Duration::from_secs(2)))
		.build()
		.expect("Failed to create the connection pool for Postgres")
}
