use super::*;

mod binance;

pub enum Interceptor {
    GRPC,
    Redis,
    Websockets
}