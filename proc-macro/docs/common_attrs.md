- `runner/async_runner`
  - Example: `#[db(async_runner = MyRunner)]`
  - Description:
    - Sets the test runner to use for the generated test. To enforce correctness, if this is missing then `skip_test` must be specified explicitely.<br/> You can use the `default-pg-runner` or `default-sqlite-runner` features to use the [`AsyncPgRunner`](diesel_enums::AsyncPgRunner) or [`AsyncSqliteRunner`](diesel_enums::AsyncSqliteRunner) by default, or you can use the `crate-runner` or `async-crate-runner` features to specify that your default runner is located at `crate::db_test_runner::DbTestRunner`.

- `skip_test`
  - Example: `#[db(skip_test)]`
  - Description:
    - Disables the generation of a test that checks the database mapping.

- `case`
   - Example: `#[db(case = "PascalCase")]`
   - Description:
     - Defines the casing that is used for the database variants. Allowed values are: `snake_case`, `UPPER_SNAKE`, `camelCase`, `PascalCase`, `lowercase`, `UPPERCASE`, `kebab-case`. Defaults to `snake_case`.
