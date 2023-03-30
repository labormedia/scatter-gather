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
use scatter_gather::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut rng = rand::thread_rng();
    let routers = scatter_gather::routing(1_000, 32)?;
    let pool = routers.0;
    let topology = routers.1;
    let origin = pool.choose(&mut rng).unwrap();
    let destiny = pool.choose(&mut rng).unwrap();
    let mut origin_list = topology.get(&origin.clone()).expect("Cannot find peer.").to_vec();

    let mut closest = Router::from(origin.clone(), &mut origin_list).closest(*destiny);

    while &closest.0 != destiny && closest.1.ilog2() != None {
        let mut closest_list = topology.get(&closest.0.clone()).expect("Cannot find peer.").to_vec();
        closest = Router::from(closest.0.clone(), &mut closest_list).closest(*destiny);
        println!("Distance ilog2 : {:?}", closest.1.ilog2());
    }

    println!("Closest to : {:?} is {:?} with distance {:?}", destiny, closest.0, closest.1);
    Ok(())
}