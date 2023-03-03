use std::{task::{
    Poll,
    Context
}, marker::PhantomData};
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
    
    let pool_config = PoolConfig {
        task_event_buffer_size: 1
    };

    let mut grpc_pool: Pool<GrpcMiddleware<BinanceDepthInterceptor>, orderbook::Summary> = Pool::new(0, pool_config, PoolConnectionLimits::default());
    let grpc_config = ServerConfig {
        url: String::from("[::1]:54001"),
        prefix: String::from("http://"),
        init_handle: None,
        handler: PhantomData
    };
    grpc_pool.inject_connection(GrpcMiddleware::new(grpc_config));
    let value = match grpc_pool.poll(&mut Context::from_waker(futures::task::noop_waker_ref())) 
    {
        Poll::Ready(event) => { 
            #[cfg(debug_assertions)]
            println!("Poll Ready... : {event:?}");
            event
        }
        Poll::Pending => { 
            #[cfg(debug_assertions)]
            println!("Poll pending...");
            PoolEvent::Custom
        },
    } ;

    println!("Got event : {:?}", value);

}