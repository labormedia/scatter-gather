fn main() {
    prost_build::compile_protos(
        &[
            "src/keys.proto",
            // "src/envelope.proto",
            // "src/peer_record.proto",
        ],
        &["src"],
    )
    .unwrap();
}