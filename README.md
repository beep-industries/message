# Messaging Service

This repository contains a small Rust-based messaging service. The project includes an OpenAPI specification (`swagger.yaml`) and can be run locally using Docker Compose and Cargo.

## Prerequisites

- Docker and Docker Compose
- Rust toolchain (cargo)
- A copy of `.env.template` configured as needed

## Quick start

1. Copy the example environment file:

   ```bash
   cp .env.template .env
   ```

2. Start required services using Docker Compose (runs in detached mode):

   ```bash
   docker compose up -d
   ```

   This will start any configured services such as MongoDB and mongo-express.

3. Access mongo-express in your browser:

   http://localhost:8081

4. Run the application with Cargo:

   ```bash
   cargo run
   ```

   The server will start according to the configuration in the project. See `swagger.yaml` for the API specification.

## API documentation

The OpenAPI specification is provided in `swagger.yaml` at the repository root. You can use it with tools like Swagger UI or Postman to explore the API endpoints.

## Useful commands

- Copy environment template: `cp .env.template .env`
- Start services: `docker compose up -d`
- Stop services: `docker compose down`
- Run the app: `cargo run`

## Notes

- Ensure `.env` contains correct credentials and connection strings for the services started by Docker Compose.
- The repository contains an example OpenAPI spec (`swagger.yaml`) describing the endpoints, request/response models, and security scheme.

## Contributing

Please open issues or pull requests for improvements or bug fixes.
