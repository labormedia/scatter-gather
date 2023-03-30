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
    let routers = scatter_gather::routing(100_000)?;
    let pool = routers.0;
    let topology = routers.1;
    let origin = pool.choose(&mut rng).unwrap();
    let destiny = pool.choose(&mut rng).unwrap();
    let mut origin_list = topology.get(&origin.clone()).unwrap().to_vec();

    let closest = Router::from(origin.clone(), &mut origin_list).closest(*destiny);


    println!("Closest to : {:?} is {:?} with distance {:?}", destiny, closest.0, closest.1);
    Ok(())
}