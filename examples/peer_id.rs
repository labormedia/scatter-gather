use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor;
fn main() {
    for _i in 0..1000 {
        let a = PeerId::random();
        let b = PeerId::random();
        let a_key = xor::Key::from(a);
        let b_key = xor::Key::from(b);
        if let Some(distance) = &a_key.distance(&b_key).ilog2() {
            println!("{}",distance);
        };
    }
}