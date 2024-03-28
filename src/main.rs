//
// use scatter_gather_core::{
//     middleware_specs::{
//         NodeConfig, Interceptor,
//     },
//     pool::{
//         Pool,
//         PoolConfig,
//         PoolConnectionLimits, 
//     },
//     connection::ConnectionHandlerOutEvent
// };
// use std::task::Poll;
use rand::seq::SliceRandom;
use buckets::peer_id::PeerId;
use scatter_gather::Router;
use rayon::{
    prelude::*,//{ParallelSliceMut, ParallelBridge},
    iter::{
        IntoParallelRefIterator,
        ParallelIterator,
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    println!("Generating Distributed Hash Table.");
    let mut rng = rand::thread_rng();
    const NETWORK_SIZE: usize = 1_000_000;
    const ROUTER_SIZE: usize = 7;
    let collection: Vec<PeerId> = (0..NETWORK_SIZE)
        .into_par_iter()
        .map(|_| {
            PeerId::random()
        })
        .collect();
    let origin = collection.choose(&mut rng).unwrap();
    let target = collection.choose(&mut rng).unwrap();
    let dht = scatter_gather::DHT::new().routing(collection.to_vec(), ROUTER_SIZE)?;
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
    println!("Closests to {target:?} from {origin:?} : {:?}", closest[0]);
    Ok(())
}