use super::*;
use std::{env, time::Duration};

use deadpool_diesel::{
	ManagerConfig, RecyclingMethod, Runtime,
	sqlite::{Hook, HookError, Manager, Pool},
};
use deadpool_sync::SyncWrapper;
use diesel::{SqliteConnection, prelude::*};
use dotenvy::dotenv;
use tokio::sync::OnceCell;

static SQLITE_POOL: OnceCell<Pool> = OnceCell::const_new();

/// The default (async) test runner for SQLite. It uses `deadpool-diesel` to create a connection pool, and sets the journal mode to WAL to allow for concurrent reads and faster tests.
///
/// It requires setting the env `DATABASE_URL` (via regular env or `.env` file) to set up the connection pool.
///
/// If you want the [`DbEnum`] macro invocations to use this as the `async_runner` by default, you can use the `default-pg-runner` feature.
pub struct AsyncSqliteRunner;

impl AsyncTestRunner<SqliteConnection> for AsyncSqliteRunner {
	async fn run_check<F>(f: F) -> Result<(), DbEnumError>
	where
		F: FnOnce(&mut SqliteConnection) -> Result<(), DbEnumError> + Send + 'static,
	{
		SQLITE_POOL
			.get_or_init(|| async { create_sqlite_pool() })
			.await
			.get()
			.await
			.expect("Failed to get a connection to the SQLite database")
			.interact(f)
			.await
			.expect("SQLite testing pool thread crashed")
	}
}

fn create_sqlite_pool() -> Pool {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL")
		.expect("Failed to set up testing pool for SQLite: DATABASE_URL is not set");

	{
		use diesel::Connection;
		let mut setup_conn = diesel::sqlite::SqliteConnection::establish(&database_url)
			.expect("Failed to connect to SQLite for initial setup");

		// Enable WAL before we create the connections in the pool to avoid locking issues
		diesel::sql_query("PRAGMA journal_mode = WAL;")
			.execute(&mut setup_conn)
			.expect("Failed to enable WAL");
	}

	let manager_config = ManagerConfig {
		recycling_method: RecyclingMethod::Fast,
	};

	let manager = Manager::from_config(database_url, Runtime::Tokio1, manager_config);

	Pool::builder(manager)
		.max_size(16)
		.runtime(Runtime::Tokio1)
		.create_timeout(Some(Duration::from_secs(5)))
		.wait_timeout(Some(Duration::from_secs(5)))
		.post_create(Hook::async_fn(move |conn, _metrics| {
			Box::pin(connection_setup(conn))
		}))
		.build()
		.expect("Failed to build the connection pool for SQLite")
}

async fn connection_setup(conn: &mut SyncWrapper<SqliteConnection>) -> Result<(), HookError> {
	conn.interact(move |conn| {
		diesel::sql_query("PRAGMA synchronous = NORMAL;").execute(conn)?;
		diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(conn)?;
		diesel::sql_query("PRAGMA mmap_size = 134217728;").execute(conn)?;
		diesel::sql_query("PRAGMA cache_size = 2000;").execute(conn)?;
		diesel::sql_query("PRAGMA foreign_keys = ON;").execute(conn)?;
		QueryResult::Ok(())
	})
	.await
	.map_err(|interact_error| HookError::Message(interact_error.to_string().into()))?
	.map_err(|query_error| HookError::Message(query_error.to_string().into()))?;

	Ok(())
}
