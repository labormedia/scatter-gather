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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut rng = rand::thread_rng();
    let routers = scatter_gather::routing(100_000)?;
    let pool = routers.0;
    let topology = routers.1;
    let origin = pool.choose(&mut rng).unwrap();
    let _destiny = pool.choose(&mut rng).unwrap();
    let a = topology.get(origin).unwrap().to_vec();
    println!("a : {:?}", a);
    Ok(())
}