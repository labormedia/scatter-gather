use scatter_gather::source_specs::binance::BinanceDepthInterceptor;
use scatter_gather_core::{
    middleware_specs::{
        ServerConfig,
    },
    pool::{
        PoolConfig,
        Pool,
        PoolConnectionLimits
    }
};
use scatter_gather_grpc::GrpcMiddleware;
mod source_specs;
pub mod orderbook {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

fn main() {
    let binance_interceptor = source_specs::binance::BinanceDepthInterceptor::new();

    let grpc_config = ServerConfig {
        url: String::from(""),
        prefix: String::from(""),
        init_handle: None,
        handler: binance_interceptor
    };

    let pool_config = PoolConfig {
        task_event_buffer_size: 1
    };

    GrpcMiddleware::new(grpc_config);
    let grpc_pool: Pool<GrpcMiddleware<BinanceDepthInterceptor>, orderbook::Summary> = Pool::new(0, pool_config, PoolConnectionLimits::default());

}