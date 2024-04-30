use rand::seq::SliceRandom;
use identity_buckets::{
    peer_id::PeerId,
    dht::{
        DHT,
        Router
    }
};
#[cfg(feature="rayon")]
use rayon::{
    prelude::*,//{ParallelSliceMut, ParallelBridge},
    iter::{
        IntoParallelRefIterator,
        ParallelIterator,
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    println!("Generating Distributed Hash Table.\n This could take some time depending on the computational resources.");
    let mut rng = rand::thread_rng();
    const NETWORK_SIZE: usize = 5_000_000;
    const ROUTER_SIZE: usize = 20;
    #[cfg(feature="rayon")]
    let collection: Vec<PeerId> = (0..NETWORK_SIZE)
        .par_bridge()
        .map(|_| {
            PeerId::random()
        })
        .collect();
    #[cfg(not(feature="rayon"))]
    let collection: Vec<PeerId> = (0..NETWORK_SIZE)
        .map(|_| {
            PeerId::random()
        })
        .collect();
    let origin = collection.choose(&mut rng).unwrap();
    let target = collection.choose(&mut rng).unwrap();
    let dht = DHT::new().routing(&collection, ROUTER_SIZE)?;
    let origin_list = match dht.routes.get(&origin.clone()) {
        Some(value) => {
            value.clone()
        },
        None => panic!("no initial routing.")
    };
    println!("DHT generated.");
    let initial_peer = Router::from(*origin, origin_list).closest(*target);
    let initial_list = dht.routes
        .get(&initial_peer.0)
        .expect("Cannot find peer.")
        .to_vec();
    let router = Router::from(initial_peer.0, initial_list);
    const K: usize = 3;
    let mut closest = router.k_closest(*target, K);
    #[cfg(feature="rayon")]
    while closest[0].1.ilog2() != None {
        closest = closest
            .par_iter()
            .flat_map(|x| {
                Router::from(x.0, dht
                    .routes
                    .get(&x.0)
                    .expect("Couldn't find node.")
                    .clone())
                    .k_closest(*target, K)
            })
            .collect();
        closest.par_sort();
        println!("Closest: {:?}", closest[0]);
    }
    #[cfg(not(feature="rayon"))]
    while closest[0].1.ilog2() != None {
        closest = closest
            .iter()
            .flat_map(|x| {
                Router::from(x.0, dht
                    .routes
                    .get(&x.0)
                    .expect("Couldn't find node.")
                    .clone())
                    .k_closest(*target, K)
            })
            .collect();
        closest.sort();
        println!("Closest: {:?}", closest[0]);
    }
    println!("Closests to {target:?} from {origin:?} : {:?}", closest[0]);
    Ok(())
}