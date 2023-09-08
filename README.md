![Edgeworth](Untitled.png "Edgeworth cage.")
# scatter-gather
A distributed system simulation library with a scatter-gather approach.

# Build
```
cargo build --all --examples
```
# Run tests + bench
```
cargo test --examples -- --nocapture
```
# List of examples to run
```
cargo run --example
cargo run --example pool-to-grpc
```
# Build example project.
```
cargo build --release --bins
```
# Run example gRPC server (optimized).
```
cargo run --release --example ws-to-grpc_example
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
