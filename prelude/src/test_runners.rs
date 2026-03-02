#[cfg(feature = "sqlite-async-runner")]
mod sqlite;

#[cfg(feature = "sqlite-async-runner")]
pub use sqlite::AsyncSqliteRunner;

#[cfg(feature = "pg-async-runner")]
mod postgres;

#[cfg(feature = "pg-async-runner")]
pub use postgres::AsyncPgRunner;

use crate::*;

/// The outcome of an operation that checks if a Rust enum and a database source are mapped correctly.
pub type DbEnumCheck = Result<(), DbEnumError>;

/// Trait for running database checks for mapped enums, synchronously.
///
/// The only required method is [`run_check`](SyncTestRunner::run_check), which passes the database connection to the test runner specified with the `sync_runner` attribute in a [`PgEnum`] or [`DbEnum`] macro invocation.
///
/// To avoid passing the runner as an attribute for each invocation, you can use the `crate-runner` feature, which will make the macro look for a type implementing [`SyncTestRunner`] at `crate::db_test_runner::DbTestRunner` by default.
pub trait SyncTestRunner<Conn>
where
	Conn: diesel::Connection,
{
	fn run_check<F>(f: F) -> DbEnumCheck
	where
		F: FnOnce(&mut Conn) -> DbEnumCheck;

	/// Checks if an enum mapping between database and Rust is correct.
	#[track_caller]
	fn check_enum<'query, E: DbEnum>() -> DbEnumCheck
	where
		<E::Table as SelectDsl<(E::IdColumn, E::NameColumn)>>::Output:
			LoadQuery<'query, Conn, (E::IdType, String)>,
	{
		Self::run_check(|conn| E::check_db_mapping(conn))
	}

	/// Checks if an enum mapping between a postgres enum and a Rust enum is correct.
	#[cfg(feature = "postgres")]
	fn check_pg_enum<E: PgEnum>() -> DbEnumCheck
	where
		Conn: diesel::Connection<Backend = diesel::pg::Pg>,
		Conn: LoadConnection,
	{
		Self::run_check(|conn| E::check_db_mapping(conn))
	}
}

/// Trait for running database checks for mapped enums, asynchronously.
///
/// The only required method is [`run_check`](AsyncTestRunner::run_check), which passes the database connection to the test runner.
///
/// To avoid passing the runner as an attribute for each invocation, you can use the `async-crate-runner` feature, which will make the macro look for a type implementing [`AsyncTestRunner`] at `crate::db_test_runner::DbTestRunner` by default, or you can use the `default-sqlite-runner` or `default-pg-runner` to use the [`AsyncSqliteRunner`] or [`AsyncPgRunner`] instead.
pub trait AsyncTestRunner<Conn>
where
	Conn: diesel::Connection + 'static,
{
	fn run_check<F>(f: F) -> impl Future<Output = DbEnumCheck> + Send
	where
		F: FnOnce(&mut Conn) -> DbEnumCheck + Send + 'static;

	/// Checks if an enum mapping between database and Rust is correct.
	#[track_caller]
	fn check_enum<'query, E: DbEnum>() -> impl Future<Output = DbEnumCheck>
	where
		<E::Table as SelectDsl<(E::IdColumn, E::NameColumn)>>::Output:
			LoadQuery<'query, Conn, (E::IdType, String)>,
	{
		Self::run_check(|conn| E::check_db_mapping(conn))
	}

	/// Checks if an enum mapping between a postgres enum and a Rust enum is correct.
	#[cfg(feature = "postgres")]
	fn check_pg_enum<E: PgEnum>() -> impl Future<Output = DbEnumCheck> + Send
	where
		Conn: diesel::Connection<Backend = diesel::pg::Pg>,
		Conn: LoadConnection,
	{
		Self::run_check(|conn| E::check_db_mapping(conn))
	}
}
