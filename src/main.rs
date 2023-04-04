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
use scatter_gather_models::{
    peer_id::PeerId,
    xor::Distance
};
use scatter_gather::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut rng = rand::thread_rng();
    let network_size = 1_000;
    let mut collection: Vec<PeerId> = Vec::new();
    for _i in 0..network_size {
        collection.push(
                PeerId::random()
        );
    }
    let dht = scatter_gather::DHT::new().routing(collection.clone(), 20)?;
    let origin = collection.choose(&mut rng).unwrap();
    let target = collection.choose(&mut rng).unwrap();
    let origin_list = match dht.routes.get(&origin.clone()) {
        Some(value) => {
            value
        },
        None => panic!("no initial routing.")
    };

    let initial_peer = Router::from(origin.clone(), origin_list.clone()).closest(*target);
    let initial_list = dht.routes.get(&initial_peer.0).expect("Cannot find peer.");
    let router = Router::from(initial_peer.0, initial_list.clone());
    let closest = router.k_closest(*target, 3);
    println!("Closests to {target:?} from {origin:?} : {closest:?}");
    Ok(())
}