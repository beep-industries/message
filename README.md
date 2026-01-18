# Messages service

The Communities service is designed to facilitate the creation, management, and interaction of user communities within the platform.
It will handle:

- Messages
- Members
- Roles
- Channels

## Prerequisites

- [Docker](https://www.docker.com/get-started/)
- Rust and Cargo
- [sqlx-cli](https://crates.io/crates/sqlx-cli)

## Quickstart

Launch postgres:

```bash
docker compose up -d mongo 
```

Create the .env file to let the Mongo client know how to connect to the database:

```bash
cp .env.example .env
```

Launch the API server:

```bash
cargo run --bin api
```

The application runs two servers on separate ports:

- **Health server** on `http://localhost:9090` - Isolated health checks (prevents DDOS on API)
  - `GET /health` - Health check with database connectivity
- **API server** on `http://localhost:3001` - Main application endpoints
  - Future business logic endpoints will be added here

This dual-server architecture provides DDOS protection by isolating health checks from API traffic.

## Configuration

You can pass down some configuration using `--help`:

```bash
cargo run --bin api -- --help
```

You can now see all the possible way to configure the service:

```bash
Communities API Server

Usage: api [OPTIONS] --database-password <database_password> --jwt-secret-key <jwt_secret_key>

Options:
      --database-rui <URI>
          [env: DATABASE_URI=] [default: mongodb://localhost:27017/messages]
      --database-name <database_name>
          [env: DATABASE_NAME=] [default: communities]
      --jwt-secret-key <jwt_secret_key>
          [env: JWT_SECRET_KEY=a-string-secret-at-least-256-bits-long]
      --server-api-port <api_port>
          [env: API_PORT=3001] [default: 8080]
      --server-health-port <HEALTH_PORT>
          [env: HEALTH_PORT=9090] [default: 8081]
  -h, --help
          Print help
```

## Persistence

To persist data we use MongoDB.

## Testing

This repository includes unit and integration tests across the core and API layers.

- Run all tests for the core crate:

```bash
cargo test
```

- Run a single test binary (example):

```bash
cargo test -p communities_core --test mongo_repo_integration -- --nocapture
```

Integration tests that exercise the MongoDB-backed repository will automatically try to start a temporary
MongoDB container using the `docker` CLI if no `MONGO_TEST_URI` environment variable is provided.
This means in most cases you can run `cargo test` without additional setup as long as Docker is installed
and the current user can run Docker commands.

If you prefer to run tests against an existing MongoDB instance, set the following environment variables:

- `MONGO_TEST_URI` — connection URI (e.g. `mongodb://localhost:27017`)
- `MONGO_TEST_DB` — optional database name (defaults to `message_test_db`)

Example (use local Mongo instance):

```bash
export MONGO_TEST_URI='mongodb://localhost:27017'
export MONGO_TEST_DB='message_test_db'
cargo test -p communities_core --test mongo_repo_integration -- --nocapture
```

If Docker is not available and `MONGO_TEST_URI` is not set, the Mongo integration test will be skipped so the
test suite still runs.

Where tests live:

- `core/tests/` — unit and integration tests for the core business logic and repositories
- `api/tests/` — HTTP integration tests for the API handlers

If you run into permission errors when starting Docker containers from tests, make sure your user is in the
`docker` group or run the tests from an environment where the Docker daemon is reachable.
