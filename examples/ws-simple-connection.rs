use scatter_gather_core::{
    middleware_specs::{
        ServerConfig,
        Interceptor
    }
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather::source_specs::binance::BinanceDepth;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    struct BinanceInterceptor {}

    #[derive(Debug)]
    enum CustomBinanceInEvent {
        Message(String),
        Error(Box<dyn std::error::Error + Send>)
    }

    impl BinanceInterceptor {
        fn new() -> Self {
            Self {}
        }
    }

    impl Interceptor for BinanceInterceptor {
        type Input = BinanceDepth;

        type InterceptorError = ();
        type InterceptorInEvent = CustomBinanceInEvent;
        type InterceptorOutEvent = ();

        fn inject_event(&mut self, event: Self::InterceptorInEvent) {
            ()
        }
    }

    let binance_interceptor = BinanceInterceptor::new();

    let config: ServerConfig<BinanceInterceptor> = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/bnbbtc@depth@100ms"),
        prefix: String::from(""),
        interceptor: binance_interceptor
    };
    let mut ws_stream = WebSocketsMiddleware::new(config).connect().await;
    while let Some(a) = ws_stream.next().await {
        println!("test {:?}", a);
    }
    Ok(())
}