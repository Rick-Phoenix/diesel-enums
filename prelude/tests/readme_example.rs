use diesel_enums::db_enum;
use std::{error::Error, time::Duration};

use deadpool_diesel::{
	Runtime,
	sqlite::{Manager, Object, Pool},
};
use diesel::{SqliteConnection, connection::SimpleConnection, prelude::*};
use diesel_enums::AsyncTestRunner;
use tokio::sync::OnceCell;

// Normally these would be in the `schema.rs` file...
diesel::table! {
	statuses (id) {
		id -> Integer,
		name -> Text
	}
}

diesel::table! {
	users (id) {
		id -> Integer,
		name -> Text,
		status_id -> Integer,
	}
}

diesel::joinable!(users -> statuses (status_id));
diesel::allow_tables_to_appear_in_same_query!(statuses, users,);

// We set up a custom runner to use in the generated tests
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

#[db_enum]
// Using our custom test runner
#[db(async_runner = SqliteRunner, table = statuses)]
pub enum Status {
	Offline,
	Active,
	Busy,
}

#[derive(
	Clone, Debug, PartialEq, Eq, Queryable, Selectable, Insertable, Associations, Identifiable,
)]
#[diesel(belongs_to(Status))]
pub struct User {
	#[diesel(skip_insertion)]
	pub id: i32,
	pub name: String,
	pub status_id: Status,
}

static SQLITE_POOL: OnceCell<Pool> = OnceCell::const_new();

pub async fn get_or_init_pool() -> Object {
	SQLITE_POOL
		.get_or_init(create_sqlite_pool)
		.await
		.get()
		.await
		.expect("Could not get a connection")
}

pub async fn create_sqlite_pool() -> deadpool_diesel::sqlite::Pool {
	let db_url = "file:example_db?mode=memory&cache=shared";
	let manager = Manager::new(db_url, Runtime::Tokio1);

	let pool = Pool::builder(manager)
		.max_size(1)
		.runtime(Runtime::Tokio1)
		.wait_timeout(Some(Duration::from_secs(5)))
		.create_timeout(Some(Duration::from_secs(5)))
		.build()
		.expect("could not build the connection pool");

	pool.get()
		.await
		.expect("Failed to get connection")
		.interact(|conn| {
			conn.batch_execute(
				r"
			PRAGMA foreign_keys = ON;

			CREATE TABLE statuses (
				id INTEGER NOT NULL PRIMARY KEY autoincrement,
				name TEXT NOT NULL
			);

			INSERT INTO statuses (name) VALUES
				('offline'), ('active'), ('busy');

			CREATE TABLE users (
				id INTEGER NOT NULL PRIMARY KEY autoincrement,
				name TEXT NOT NULL,
				status_id INTEGER NOT NULL,
				FOREIGN KEY (status_id) REFERENCES statuses (id)
			);
		",
			)
		})
		.await
		.expect("Failed interaction")
		.expect("Failed initial query");

	pool
}

pub async fn run_sqlite_query<T: Send + 'static>(
	callback: impl FnOnce(&mut SqliteConnection) -> QueryResult<T> + Send + 'static,
) -> Result<T, Box<dyn Error>> {
	Ok(get_or_init_pool()
		.await
		.interact(callback)
		.await??)
}

#[tokio::test]
async fn example() {
	let tom = User {
		id: 1,
		name: "Tom Bombadil".to_string(),
		status_id: Status::Active,
	};
	let clone = tom.clone();

	let inserted = run_sqlite_query(move |conn| {
		diesel::insert_into(users::table)
			.values(&clone)
			.returning(User::as_select())
			.get_result(conn)
	})
	.await
	.expect("Failed insertion");

	assert_eq!(inserted, tom);

	let filtered_query = run_sqlite_query(|conn| {
		users::table
			.select(User::as_select())
			// We can filter with the enum directly!
			.filter(users::status_id.eq(Status::Active))
			.first(conn)
	})
	.await
	.expect("Failed filtered query");

	assert_eq!(filtered_query, tom);

	let all_active = run_sqlite_query(|conn| {
		// Join queries become very concise
		User::belonging_to(&Status::Active)
			.select(User::as_select())
			.get_results(conn)
	})
	.await
	.expect("Failed query");

	assert_eq!(all_active.first().unwrap(), &tom);
}

#[db_enum]
// Wrong mapping! The generated test would catch this
#[db(skip_test, table = statuses)]
pub enum WrongStatus {
	Offline,
	Active,
}

#[tokio::test]
async fn wrong_status() {
	assert!(
		SqliteRunner::check_enum::<WrongStatus>()
			.await
			.is_err()
	)
}
