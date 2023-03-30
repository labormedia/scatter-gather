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
use rand::seq::IteratorRandom;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut rng = rand::thread_rng();
    let routers = scatter_gather::routing(100_000);
    Ok(())
}