## [0.2.1] - 2026-03-02

### 📚 Documentation

- [8c7045c](https://github.com/Rick-Phoenix/diesel-enums/commit/8c7045cb613d365fd4cd2c5b4d3be150e0e9c2b6) Fix docs.rs failed build

## [0.2.0] - 2026-03-02

### ⛰️  Features

- [773a3ef](https://github.com/Rick-Phoenix/diesel-enums/commit/773a3ef2d8c169aa8edf33cfde578bc3cf63a0fd) Added case conversion and made check optional
- [2225eab](https://github.com/Rick-Phoenix/diesel-enums/commit/2225eab4610b894a91bdaf273c5696d61227884b) Added crate feature for default check behaviour
- [c4a9451](https://github.com/Rick-Phoenix/diesel-enums/commit/c4a945129909afbf8b6fa88b4182fcf5f5334035) Allowing mapping of different types
- [96f66af](https://github.com/Rick-Phoenix/diesel-enums/commit/96f66afec262f272f9bfdd3525945c0369ce7395) Auto implementation for auto incrementing integers
- [7469025](https://github.com/Rick-Phoenix/diesel-enums/commit/74690257c005dd6255ae54dc0061125a9274f53c) Feature flag for default auto increment
- [85c04f7](https://github.com/Rick-Phoenix/diesel-enums/commit/85c04f7c538c347e13b2e8b9a750d9f46a5e0d88) Automatic test for custom postgres enums
- [648e999](https://github.com/Rick-Phoenix/diesel-enums/commit/648e9993eedbbdfd9f1c4f5e2ba219123d4de815) Added prettified error messages
- [4ccde8a](https://github.com/Rick-Phoenix/diesel-enums/commit/4ccde8aa0fda3519f202dbb2e2707113e02e5eda) Create separate method for manual consistency check
- [9120e71](https://github.com/Rick-Phoenix/diesel-enums/commit/9120e7187f361cec8304dcf462efc1d58b7dc093) Separate check and test generation and allow skipping of one or the two
- [147dec0](https://github.com/Rick-Phoenix/diesel-enums/commit/147dec072ad0a750713acff4ef7981b8fb4578bb) Added ability to skip id ranges in int conversion implementation
- [a1d5903](https://github.com/Rick-Phoenix/diesel-enums/commit/a1d5903726c9761bffb50cd0278c95f99d8b1697) Added default path for connection function
- [3b2022f](https://github.com/Rick-Phoenix/diesel-enums/commit/3b2022faf6365b5aea218fffd5becf06a41131c0) Allowing the use of custom table paths
- [3434202](https://github.com/Rick-Phoenix/diesel-enums/commit/34342027f6b54326449c723a9ff6b9a8052f0b92) Added inter-call for check_consistency method when generating 2 enums
- [87c707a](https://github.com/Rick-Phoenix/diesel-enums/commit/87c707a6e83581c98860830655bae546ba21519d) Automatically implementing Copy, Clone, Eq, PartialEq and Hash for the generated enums
- [fa9e253](https://github.com/Rick-Phoenix/diesel-enums/commit/fa9e253a03b42c24247f64555de76e98870d92aa) Using the last segment of the table path in case table_name is not given, otherwise falling back to the snake_case version of the enum name
- [49a1780](https://github.com/Rick-Phoenix/diesel-enums/commit/49a17802c8f31a00940f7e7c5657843b3eae80f8) Allowing forced test generation to override defaults
- [5968834](https://github.com/Rick-Phoenix/diesel-enums/commit/5968834b8260b1e845acec8b97956bdb072ddaaf) Delegating to core crate for error creation
- [39560e6](https://github.com/Rick-Phoenix/diesel-enums/commit/39560e645c71fcdd113c46898e33bec6f9367799) Default test runners
- [5fe06e4](https://github.com/Rick-Phoenix/diesel-enums/commit/5fe06e49ac46bb949dce4f32db20433d6d454610) Default test runner for postgres
- [36e3e1a](https://github.com/Rick-Phoenix/diesel-enums/commit/36e3e1aa3d6102b0a35557649a87603069b0da51) Support for single numbers in skip_ids
- [6ae99b0](https://github.com/Rick-Phoenix/diesel-enums/commit/6ae99b0751e93a02d021b0126c2a1c751a3dedef) Added method that convers variant to str

### 🐛 Bug Fixes

- [ca31db8](https://github.com/Rick-Phoenix/diesel-enums/commit/ca31db8a1c53073f3d4c65c8b325da44a97e1585) Adjusted LitInt creation

### 🚜 Refactor

- [589f8dd](https://github.com/Rick-Phoenix/diesel-enums/commit/589f8ddcbb3a88cf0030f343dfd6e6cf42c4006f) Simplified handling for variant names and added variant attributes
- [ef5e3c1](https://github.com/Rick-Phoenix/diesel-enums/commit/ef5e3c120a2508b23867e89ffb5083bb9141c587) Map sql_type attribute to diesel's
- [a65d4ee](https://github.com/Rick-Phoenix/diesel-enums/commit/a65d4eeb3f18a428bc49bcdb875c69ae9345d086) Significantly increased clarity by linking data that must always go together
- [8e05d24](https://github.com/Rick-Phoenix/diesel-enums/commit/8e05d2416df7ada8f4141432b74b497f91ce295a) Renamed features for consistency with macro attributes
- [141cf22](https://github.com/Rick-Phoenix/diesel-enums/commit/141cf225c86effa6f27fd87ad6622c4a7084a701) Accepting a callback to get the connection in tests to allow for the usage of pools
- [370bbd2](https://github.com/Rick-Phoenix/diesel-enums/commit/370bbd292c1fc0156cfcdc3d43cf15766cfc0af2) Renamed skip_check to skip_consistency_check to differentiate it from skip_test
- [657df07](https://github.com/Rick-Phoenix/diesel-enums/commit/657df07fee7eeea47aabeee447b3fa7e0bfae1cd) Removed skippable variants
- [2b72a31](https://github.com/Rick-Phoenix/diesel-enums/commit/2b72a315e2c39a055e1752206f5c9d292e7f9e61) Only generating check_consistency for the non-id enum in case of double mapping

### 📚 Documentation

- [6452adc](https://github.com/Rick-Phoenix/diesel-enums/commit/6452adc630fb98ab46dae80b823712124c8eb72c) Added documentation
- [318c074](https://github.com/Rick-Phoenix/diesel-enums/commit/318c0745fbfd8a2849fb4edb08eecf4559e35341) Added documentation for exposed functions and items
- [f03771c](https://github.com/Rick-Phoenix/diesel-enums/commit/f03771c7c6df006eceaa2906f012429ed5b6a5cd) Separated query test to use it as an example
- [7081c47](https://github.com/Rick-Phoenix/diesel-enums/commit/7081c47e6a86ec23c40c88f6f0038f11e3c71b1d) Added postgres example
- [3e40859](https://github.com/Rick-Phoenix/diesel-enums/commit/3e408593d3020465473389e14222997ddc9a3737) Populated main documentation page

### 🧪 Testing

- [37ab3fe](https://github.com/Rick-Phoenix/diesel-enums/commit/37ab3fe212426ad1d3ef98bf6feb2521933f1e03) Added testing crate
- [903842d](https://github.com/Rick-Phoenix/diesel-enums/commit/903842dfdd15f1800c711a53cb19baaf09d1910e) Refined tests with new detailed errors

