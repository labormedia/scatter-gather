use futures::StreamExt;
use scatter_gather_core::{
    middleware_specs::{
        ServerConfig, Interceptor,
    },
    pool::{
        Pool,
        PoolConfig,
        PoolConnectionLimits, 
    },
    connection::ConnectionHandlerOutEvent
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather_grpc::{
    GrpcMiddleware,
    schema_specific::{
        orderbook::{
            Summary, 
            Level
        }
    }
};
use scatter_gather::source_specs::{
    Depth,
    Interceptors,
    binance::BinanceDepthInterceptor,
    bitstamp::BitstampDepthInterceptor,
};
use std::task::Poll;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    Ok(())
    
}