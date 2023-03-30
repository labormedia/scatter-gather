use std::collections::HashMap;

use rand::seq::{SliceRandom, IteratorRandom};
use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    generating_random(100_000);
    sorting(1_000_000)?;
    generating_sequential(99_999_000)?;
    let routers = routing(100_000);
    println!("Generating routers, i.e. {:?}", routers);
    Ok(())
}

fn sorting(size: usize) -> Result<Vec<PeerId>, Box<dyn std::error::Error>> {
    let mut collection: Vec<PeerId> = Vec::new();
    for _i in 0..size {
        collection.push(PeerId::random());
    }
    let a = collection.sort();
    let b: Vec<PeerId> = collection.iter().take(100).map(|x| { println!("{}", x); *x } ).collect();
    Ok(collection)
}

fn generating_random(size: usize) {
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

fn generating_sequential(size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut count_a = 0;
    let mut count_b = 0;
    let first = PeerId::random();
    for _i in 0..size as u8 {
        let first_key = xor::Key::from(first);
        let a = PeerId::random();
        let b = PeerId::random();
        let a_key = xor::Key::from(a);
        let b_key = xor::Key::from(b);
        let distance_first_a = first_key.distance(&a_key);
        let distance_first_b= first_key.distance(&b_key);
        let compare = distance_first_a < distance_first_b;
        if compare == true { count_a +=1; } else { count_b +=1; }
    }
    println!("Distribution: {:?} and {:?} of {:?}", count_a, count_b, size);
    Ok(())
}

#[derive(Clone, Debug)]
struct Router {
    peer_id: PeerId,
    bucket: Vec<PeerId>
}

fn routing(size: usize) -> Result<Vec<PeerId>, Box<dyn std::error::Error>> {
    let mut collection: Vec<PeerId> = Vec::new();
    for _i in 0..size {
        collection.push(
                PeerId::random()
        );
    }
    let cloned_collection = collection.clone();
    let new_collection = collection.
        into_iter()
        .fold( HashMap::new(),
            |mut a: HashMap<PeerId, Vec<PeerId>>, x: PeerId|
            {
                a.insert(x, cloned_collection
                    .choose_multiple(&mut rand::thread_rng(), 17)
                    .cloned()
                    .collect());
                a
            }
            
        );
    let origin = cloned_collection.choose(&mut rand::thread_rng()).unwrap();
    let origin_list = new_collection.get(origin).unwrap().to_vec();

    // let destiny = cloned_collection.choose(&mut rand::thread_rng()).unwrap();



    Ok(origin_list)
}