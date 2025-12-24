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
