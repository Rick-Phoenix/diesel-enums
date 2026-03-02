#[cfg(test)]
mod tests {
	mod models;
	mod pg_example;
	mod pg_tests;
	mod schema;

	use deadpool_diesel::{
		Runtime,
		postgres::{Manager as PgManager, Pool as PgPool},
	};
	use diesel::prelude::*;
	use diesel_enums::AsyncTestRunner;
	use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
	use pgtemp::PgTempDB;
	use tokio::sync::OnceCell;

	const PG_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

	pub(crate) static POSTGRES_POOL: OnceCell<deadpool_diesel::postgres::Pool> =
		OnceCell::const_new();

	pub(crate) struct PgRunner;

	impl AsyncTestRunner<PgConnection> for PgRunner {
		async fn run_check<F>(f: F) -> diesel_enums::DbEnumCheck
		where
			F: FnOnce(&mut PgConnection) -> diesel_enums::DbEnumCheck + Send + 'static,
		{
			POSTGRES_POOL
				.get_or_init(|| async { create_pg_pool().await })
				.await
				.get()
				.await
				.expect("Could not get a connection")
				.interact(f)
				.await
				.expect("Testing outcome was unsuccessful")
		}
	}

	// Needs to be put here to avoid being dropped earlier
	static PG_TEMP: OnceCell<PgTempDB> = OnceCell::const_new();

	pub async fn create_pg_pool() -> deadpool_diesel::postgres::Pool {
		let db = PG_TEMP
			.get_or_init(async || PgTempDB::async_new().await)
			.await;

		let url = db.connection_uri();

		let manager = PgManager::new(url, Runtime::Tokio1);

		let pool = PgPool::builder(manager)
			.max_size(1)
			.runtime(Runtime::Tokio1)
			.build()
			.expect("could not build the postgres connection pool");

		pool.get()
			.await
			.unwrap()
			.interact(|conn| {
				conn.run_pending_migrations(PG_MIGRATIONS)
					.expect("Failed to run migrations");
			})
			.await
			.unwrap();

		pool
	}
}
