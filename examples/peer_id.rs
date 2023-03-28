use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor;
fn main() {
    generating(100_000);
    sorting(100_000);
}

fn sorting(size: usize) {
    let mut collection: Vec<PeerId> = Vec::new();
    for _i in 0..size {
        collection.push(PeerId::random());
    }
    let a = collection.sort();
    let b: Vec<PeerId> = collection.iter().take(100).map(|x| { println!("{}", x); *x } ).collect();
}

fn generating(size: usize) {
    let mut count = 0;
    for _i in 0..size {
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
    println!("Distribution: {:?} of {:?}", count, size);
}