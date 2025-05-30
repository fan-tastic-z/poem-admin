# poem-admin

A Rust-based admin system using Poem web framework.

## Features

- JWT-based authentication system
- Role-based access control with Casbin
- PostgreSQL database with SQLx
- User, role, and organization management
- Operation logging
- RESTful API with Poem framework

## Development

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Docker (for integration tests)

### Quick Start

1. Clone the repository:

```bash
git clone git@github.com:fan-tastic-z/poem-admin.git
```

Set up the database:

```bash
# Create database and run migrations
cargo install sqlx-cli --no-default-features --features postgres
./scripts/init_db.sh
```

Init base data:

```bash
cargo run --bin poem-admin init-data -c ./dev/config.toml
```

Create super user

```bash
cargo run --bin poem-admin create-super-user -c ./dev/config.toml -p 12345678
```

Start server

```bash
cargo run --bin poem-admin server -c ./dev/config.toml
```

I use AI generate front-end code

```bash
git clone git@github.com:fan-tastic-z/admin-web.git
```

Start front-end

```bash
pnpm dev
```

### Testing

#### Unit Tests

```bash
cargo test --lib
```

#### Integration Tests

```bash
# Requires Docker to be running
cargo test --test integration_tests
cargo test --test api_integration_tests
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
