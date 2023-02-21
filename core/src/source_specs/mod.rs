// #[derive(Serialize, Deserialize)]
// struct dummy {
//     i: i32
// }
use super::*;

mod binance;

pub enum Interceptor {
    GRPC,
    Redis,
    Websockets
}

pub struct ServerConfig {
    url: String,
    prefix: String,
    protocol: Interceptor,
    executor: Option<Box<dyn Executor + Send>>
}