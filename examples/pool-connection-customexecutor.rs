use scatter_gather_core::{
    middleware_interface::{
        NodeConfig,
        GetStream,
    },
    pool::{
        Pool,
        PoolConfig,
        PoolConnectionLimits,
    },
    executors::CustomExecutor,
};
use scatter_gather_websockets::WebSocketsMiddleware;
use tungstenite::Message;
use std::sync::mpsc::sync_channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    type WSPool = Pool<WebSocketsMiddleware,Result<Message, tungstenite::Error>, usize>;

    let mut ws_pool: WSPool = Pool::new(
        0_usize, // Pool ID
        PoolConfig { // Pool General Configuration
            task_event_buffer_size: 1
        }, 
        PoolConnectionLimits::default() // Pool Limits
    );

    const MAX_QUEUED_TASKS: usize = 10;
    let (executor, receiver) = sync_channel(MAX_QUEUED_TASKS);
    let custom_executor = CustomExecutor::with_sender(executor);

    ws_pool.with_executor(
        Box::new(custom_executor)
    );

    let _ = [ // Slice of WebSocket connection configurations.
        NodeConfig::from(
            "wss://stream.binance.com:9443/ws/ethbtc@depth@100ms", // url
            "wss://",  // prefix
            None,  // init handle
        ),
        NodeConfig::from( 
            "wss://ws.bitstamp.net", // url
            "wss://", // prefix
            Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#),  // init handle
        ),
    ]
    .map( |config| {
        ws_pool.inject_connection(WebSocketsMiddleware::new(config.clone()));
    } );
    
    let _ = ws_pool.connect().await;

    let mut connections = Vec::new();
    while let Ok(middleware) = receiver.try_recv() {
        connections.push(middleware.await);
    }

    loop {
        let mut i = 0;
        for mut conn in &mut connections {
            if let Some(data) = conn.next().await {
                #[cfg(debug_assertions)]
                println!("Incoming data (Stream {}) {:?}", i, data);
            };
            i = 1;
        }
    }

    Ok(())
}