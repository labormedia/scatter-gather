![Edgeworth](Untitled.png "Edgeworth cage.")
# scatter-gather
A distributed system simulation library with a scatter-gather approach.

# Build
```
cargo build --release --all --examples
```
# Run tests + bench (requires "ws-to-grpc_server" example running)
```
cargo test --release --examples -- --nocapture
```
# List of examples to run
```
cargo run --release --example
cargo run --release --example pool-to-grpc
```
# Build example project.
```
cargo build --release --bins
```
# Run example gRPC server (optimized).
```
cargo run --release --example ws-to-grpc_server
```
# Run example gRPC client. (Communicates with the server)
```
cargo run --release --example ws-to-grpc_client
```

# Project Structure
.\
├── CHANGELOG.md    : List of changes throughout versions\
├── LICENSE.md      : License\
├── README.md       : This file\
├── core            : Contains the common elements for the library\
├── examples        : Example use cases\
├── middleware      : Defines specific interceptors, helpers and utilities.\
├── src             : Generic API\
└── Cargo.toml      : Project Manifest
