test-pg:
    #!/usr/bin/env sh
    PG_BIN_DIR=$(find /usr/lib/postgresql/ -type d -name "bin" | head -n 1)

    if [ -z "$PG_BIN_DIR" ]; then
      echo "Error: Could not find PostgreSQL bin directory."
      exit 1
    fi

     PATH="$PG_BIN_DIR:$PATH"  cargo test -p test-pg  -- -q 

test-sqlite:
    cargo test -p test-sqlite -- -q

test-docs:
    cargo test -p diesel-enums -- -q

test-all: test-sqlite test-pg test-docs

docs:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --open --no-deps -p diesel-enums

gen-readme:
    cargo reedme -p diesel-enums --features document-features

release version exec="": test-all gen-readme
    ./pre-release.sh {{ version }} {{ exec }}
    cargo release {{ version }} {{ exec }}
