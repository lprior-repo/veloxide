# wtf-engine

DAG-based durable workflow execution engine combining:

- **petgraph DAGs** for workflow representation
- **ractor actors** for supervision and isolation
- **sled** for embedded, async-first persistence
- **Step Functions parity** (minus long waits)
- **Single fat binary** with MIT license

## Features

- Full AWS Step Functions parity (Pass, Task, Choice, Parallel, Map, Wait, etc.)
- Journal-based replay for crash recovery
- ractor actor model with Erlang-style supervision
- petgraph-powered DAG execution
- Single binary deployment (API + Worker + Frontend + DB)
- 3x parallelism by default

## Quick Start

```bash
# Build
cargo build --release

# Run server
cargo run --release -- serve

# Run CLI
cargo run --release -- --help
```

## Documentation

- [Architecture](docs/architecture.md)
- [ADR Index](docs/adr/)

## Crates

| Crate | Description |
|-------|-------------|
| `wtf-core` | Core types, DAG, journal, replay |
| `wtf-storage` | sled persistence layer |
| `wtf-actor` | ractor actors |
| `wtf-worker` | Worker loop, activity execution |
| `wtf-api` | Axum HTTP API |
| `wtf-cli` | CLI client |
| `wtf-frontend` | Dioxus web UI |
| `wtf-common` | Shared types |

## License

MIT
