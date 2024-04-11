#[allow(clippy::derive_partial_eq_without_eq)]
pub mod keys_proto {
    include!(concat!(env!("OUT_DIR"), "/keys_proto.rs"));
}

pub mod identity;
pub mod peer_id;
pub mod xor;

#[cfg(feature="dht")]
pub mod dht;