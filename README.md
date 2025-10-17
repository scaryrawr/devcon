# devcon

Tooling for improving remote development.

This is a Rust workspace monorepo containing two applications for remote development support:

## Applications

### devcon-client

A client application that provides REST APIs and Server-Sent Events (SSE) support for managing remote development connections.

**Features:**
- REST API endpoints for client operations
- Server-Sent Events (SSE) for real-time updates
- Built with Axum web framework

**Endpoints:**
- `GET /` - Root endpoint
- `GET /api/hello` - Hello endpoint
- `POST /api/data` - Data submission endpoint
- `GET /sse` - Server-Sent Events stream

**Default Port:** 3000

### devcon-remote

A remote application that provides REST APIs, Server-Sent Events (SSE), and netlink support for discovering ports in use by applications.

**Features:**
- REST API endpoints for remote operations
- Server-Sent Events (SSE) for real-time updates
- Netlink integration for port discovery (enables port forwarding between client and remote machines)
- Built with Axum web framework

**Endpoints:**
- `GET /` - Root endpoint
- `GET /api/hello` - Hello endpoint
- `POST /api/data` - Data submission endpoint
- `GET /api/ports` - Get list of listening ports (TCP and UDP)
- `GET /sse` - Server-Sent Events stream

**Default Port:** 3001

## Building

Build all applications in the workspace:

```bash
cargo build
```

Build for release:

```bash
cargo build --release
```

## Running

Run devcon-client:

```bash
cargo run -p devcon-client
```

Run devcon-remote:

```bash
cargo run -p devcon-remote
```

Or run the release binaries:

```bash
./target/release/devcon-client
./target/release/devcon-remote
```

## Testing

Run all tests:

```bash
cargo test
```

## Linting

Format code:

```bash
cargo fmt
```

Run linter:

```bash
cargo clippy
```

## Architecture

This workspace uses:
- **Tokio** - Async runtime
- **Axum** - Web framework for REST APIs
- **Tower** - Middleware and service abstractions
- **Serde** - Serialization/deserialization
- **Tracing** - Structured logging
- **Netlink** (devcon-remote only) - Linux kernel interface for network socket diagnostics

## License

MIT

