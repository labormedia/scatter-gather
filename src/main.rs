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
use scatter_gather_models::peer_id::PeerId;
use scatter_gather::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut rng = rand::thread_rng();
    let network_size = 10_000;
    let mut collection: Vec<PeerId> = Vec::new();
    for _i in 0..network_size {
        collection.push(
                PeerId::random()
        );
    }
    let origin = collection.choose(&mut rng).unwrap().clone();
    let target = collection.choose(&mut rng).unwrap().clone();
    let dht = scatter_gather::DHT::new().routing(collection, 100)?;
    let origin_list = match dht.routes.get(&origin.clone()) {
        Some(value) => {
            value.clone()
        },
        None => panic!("no initial routing.")
    };
    println!("DHT generated.");
    let initial_peer = Router::from(origin, origin_list).closest(target);
    let initial_list = dht.routes
        .get(&initial_peer.0)
        .expect("Cannot find peer.")
        .clone()
        .to_vec();
    let router = Router::from(initial_peer.0, initial_list);
    const K: usize = 3;
    let mut closest = router.k_closest(target, K);
    while closest[0].1.ilog2() != None {
        closest = closest
            .iter()
            .flat_map(|x| {
                Router::from(x.0, dht
                    .routes
                    .get(&x.0)
                    .expect("Couldn't find node.")
                    .clone())
                    .k_closest(target, K)
            })
            .collect();
        println!("Closest: {:?}", closest[0]);
    }
    println!("Closests to {target:?} from {origin:?} : {:?}", closest[0]);
    Ok(())
}