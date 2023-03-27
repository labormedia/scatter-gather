use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor;
fn main() {
    let mut count = 0;
    let size_ids = 1000;
    for _i in 0..size_ids {
        let a = PeerId::random();
        let b = PeerId::random();
        let c = PeerId::random();
        let a_key = xor::Key::from(a);
        let b_key = xor::Key::from(b);
        let c_key = xor::Key::from(c);
        let distance_a_b = a_key.distance(&b_key);
        let distance_b_c= b_key.distance(&c_key);
        let compare = distance_a_b < distance_b_c;
        if compare == true { count +=1; }
        
    }
    println!("Distribution: {:?} of {:?}", count, size_ids);
}