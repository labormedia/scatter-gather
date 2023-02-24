use scatter_gather_core::middleware_specs::{
    ServerConfig,
    Interceptor
};
use scatter_gather_websockets::WebSocketsMiddleware;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    struct BinanceInterceptor;

    impl BinanceInterceptor {
        fn new() -> Self {
            Self
        }
    }

    impl Interceptor for BinanceInterceptor {
        type InterceptorError = ();
        type InterceptorInEvent = ();
        type InterceptorOutEvent = ();

        fn inject_event(&mut self, event: Self::InterceptorInEvent) {
            ()
        }
    }

    let binance_interceptor = BinanceInterceptor::new();

    let config: ServerConfig<BinanceInterceptor> = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/bnbbtc@depth@100ms"),
        prefix: String::from(""),
        protocol: binance_interceptor
    };
    let ws_stream = WebSocketsMiddleware::new(config).connect().await;
    let (write, read) = ws_stream.split();
    let _ = read.fold(write, |write, m| async {
        match m  {
            Err(e) => { 
                println!("error {:?}", e);
                ()
            },
            Ok(message) => {
                let _ = println!("{:?}",message);
                ()
            },
        };
        write
    }).await;
    Ok(())
}