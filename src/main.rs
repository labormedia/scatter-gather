use std::task::{
    Poll,
    Context
};
use scatter_gather::source_specs::binance::BinanceDepthInterceptor;
use scatter_gather_core::{
    middleware_specs::{
        ServerConfig,
    },
    pool::{
        PoolConfig,
        Pool,
        PoolConnectionLimits,
        PoolEvent
    }
};
use scatter_gather_grpc::GrpcMiddleware;
mod source_specs;
pub mod orderbook {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

fn main() {
    let binance_interceptor = scatter_gather::source_specs::binance::BinanceDepthInterceptor::new();

    let grpc_config = ServerConfig {
        url: String::from(""),
        prefix: String::from(""),
        init_handle: None,
        handler: binance_interceptor
    };

    let pool_config = PoolConfig {
        task_event_buffer_size: 1
    };

    let mut grpc_pool: Pool<GrpcMiddleware<BinanceDepthInterceptor>, orderbook::Summary> = Pool::new(0, pool_config, PoolConnectionLimits::default());
    let channels = GrpcMiddleware::new(grpc_config);
    grpc_pool.inject_connection(channels);
    match grpc_pool.poll(&mut Context::from_waker(futures::task::noop_waker_ref())) {
        Poll::Ready(value) => { println!("{value:?}"); }
        Poll::Pending => { println!("Poll pending."); }
    };
}