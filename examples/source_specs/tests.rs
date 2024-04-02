use core::task::{
    Poll,
    Context
};
use scatter_gather::source_specs::{binance::BinanceDepthInterceptor, bitstamp::BitstampDepthInterceptor};
use scatter_gather_core::{
    middleware_interface::{
        NodeConfig,
    },
    pool::{
        PoolConfig,
        Pool,
        PoolConnectionLimits,
        PoolEvent
    }
};
use scatter_gather_grpc::GrpcMiddleware;
use scatter_gather_websockets::WebSocketsMiddleware;
use tokio;
mod source_specs;
pub mod orderbook {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

fn main() {
    let grpc_config = NodeConfig {
        url: String::from("[::1]:54001"),
        prefix: String::from("http://"),
        init_handle: None,
        // handler: BinanceDepthInterceptor::new()
    };
    let pool_config = PoolConfig {
        task_event_buffer_size: 1
    };

    let mut grpc_pool: Pool<GrpcMiddleware, orderbook::Summary> = Pool::new(0, pool_config, PoolConnectionLimits::default());

    grpc_pool.inject_connection(GrpcMiddleware::new(grpc_config));
    grpc_pool.connect();
}