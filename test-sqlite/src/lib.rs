#[cfg(test)]
mod tests {
	mod models;
	mod schema;
	mod sqlite_example;
	mod sqlite_tests;

	use std::{env, error::Error, time::Duration};

	use deadpool_diesel::{
		Runtime,
		sqlite::{Hook, HookError, Manager as SqliteManager, Object, Pool as SqlitePool},
	};
	use deadpool_sync::SyncWrapper;
	use diesel::{SqliteConnection, prelude::*};
	use diesel_enums::AsyncTestRunner;
	use dotenvy::dotenv;
	use tokio::sync::OnceCell;

	static SQLITE_POOL: OnceCell<SqlitePool> = OnceCell::const_new();

	pub struct SqliteRunner;

	impl AsyncTestRunner<SqliteConnection> for SqliteRunner {
		async fn run_check<F>(f: F) -> diesel_enums::DbEnumCheck
		where
			F: FnOnce(&mut SqliteConnection) -> diesel_enums::DbEnumCheck + Send + 'static,
		{
			get_or_init_pool()
				.await
				.interact(f)
				.await
				.expect("Failed to interact with the database")
		}
	}

	pub async fn get_or_init_pool() -> Object {
		SQLITE_POOL
			.get_or_init(|| async { create_sqlite_pool() })
			.await
			.get()
			.await
			.expect("Could not get a connection")
	}

	pub async fn run_sqlite_query<T: Send + 'static>(
		callback: impl FnOnce(&mut SqliteConnection) -> QueryResult<T> + Send + 'static,
	) -> Result<T, Box<dyn Error>> {
		Ok(get_or_init_pool()
			.await
			.interact(callback)
			.await??)
	}

	pub fn create_sqlite_pool() -> deadpool_diesel::sqlite::Pool {
		dotenv().ok();

		let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

		let manager = SqliteManager::new(database_url, Runtime::Tokio1);

		SqlitePool::builder(manager)
			.max_size(1)
			.runtime(Runtime::Tokio1)
			.wait_timeout(Some(Duration::from_secs(5)))
			.create_timeout(Some(Duration::from_secs(5)))
			.post_create(Hook::async_fn(move |conn, _metrics| {
				Box::pin(connection_setup(conn))
			}))
			.build()
			.expect("could not build the connection pool")
	}

	async fn connection_setup(conn: &mut SyncWrapper<SqliteConnection>) -> Result<(), HookError> {
		conn.interact(move |conn| {
			diesel::sql_query("PRAGMA synchronous = NORMAL;").execute(conn)?;
			diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(conn)?;
			diesel::sql_query("PRAGMA mmap_size = 134217728;").execute(conn)?;
			diesel::sql_query("PRAGMA cache_size = 2000;").execute(conn)?;
			QueryResult::Ok(())
		})
		.await
		.map_err(|interact_error| HookError::Message(interact_error.to_string().into()))?
		.map_err(|query_error| HookError::Message(query_error.to_string().into()))?;

		Ok(())
	}
}
