#[allow(clippy::derive_partial_eq_without_eq)]
pub mod keys_proto {
    include!(concat!(env!("OUT_DIR"), "/keys_proto.rs"));
}

mod identity;
pub mod peer_id;