fn main() {
    tonic_build::compile_protos("examples/source_specs/protobuffer/orderbook.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}