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
use scatter_gather_websockets::{WebSocketsMiddleware, ConnectionHandlerError};
use scatter_gather_grpc::{
    GrpcMiddleware,
    schema_specific::{
        self, 
        OrderBook, 
        orderbook::{
            Summary, 
            Level
        }
    }
};
use scatter_gather::source_specs::{
    Depth,
    Level as DepthLevel,
    Interceptors,
    binance::BinanceDepthInterceptor,
    bitstamp::BitstampDepthInterceptor, self,
};
use tonic::service::interceptor;
use std::{
    pin::Pin,
    any::type_name,
    task::Context,
    task::Poll, iter::Sum, collections::VecDeque
};
use tokio::runtime::Runtime;

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let config_binance: ServerConfig = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/ethbtc@depth@100ms"),
        prefix: String::from("wss://"),
        init_handle: None,
        // handler: binance_interceptor
    };
    let config_bitstamp: ServerConfig = ServerConfig { 
        url: String::from("wss://ws.bitstamp.net"), 
        prefix: String::from("wss://"), 
        init_handle: Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()),
        // handler: bitstamp_interceptor
    };

    let binance = WebSocketsMiddleware::new(config_binance).await ;
    let bitstamp = WebSocketsMiddleware::new(config_bitstamp).await;

    let binance_intercepted = 
        binance
            .read
            .map(|result| result.unwrap().into_text().unwrap())
            .map(|text| Interceptors::Binance(BinanceDepthInterceptor::intercept(text)) );
    let bitstamp_intercepted =
        bitstamp
            .read
            .map(|result| result.unwrap().into_text().unwrap())
            .map(|text| Interceptors::Bitstamp(BitstampDepthInterceptor::intercept(text)) );

    let grpc_config: ServerConfig = ServerConfig { 
        url: String::from("[::1]:54001"), 
        prefix: String::from("http://"), 
        init_handle: None, 
        // handler: Interceptors::Depth
    };


    let grpc = GrpcMiddleware::new(grpc_config);

    let pool_config1 = PoolConfig {
        task_event_buffer_size: 10
    };

    let pool_config2 = PoolConfig {
        task_event_buffer_size: 10
    };

    let mut ws_pool: Pool<WebSocketsMiddleware, Interceptors> = Pool::new(0_usize, pool_config1, PoolConnectionLimits::default()); 
    let mut grpc_pool: Pool<GrpcMiddleware,Interceptors> = Pool::new(1_usize, pool_config2, PoolConnectionLimits::default());


    grpc_pool.inject_connection(grpc);


    ws_pool.collect_streams(Box::pin(binance_intercepted));
    ws_pool.collect_streams(Box::pin(bitstamp_intercepted));

    // ws_pool.local_streams
    //     .map(|interceptor| ;


    // grpc_pool.intercept_stream().await;
    // let internal_snapshot = std::time::Instant::now();Haha
    match grpc_pool.connect().await {
        Poll::Ready(conn) => {
            println!("Buffering.");
            let mut conn_lock = conn.lock().await;
            // let mut state = &conn_lock.state.clone();
            while let Some(intercepted) = ws_pool.next().await // let's bench here.
            {

                println!("State: {:?}", conn_lock.state);
                // println!("State: {:?}", conn_lock.state);
                let update_point: (Vec<Level>, Vec<Level>) = match intercepted
                {
                    Interceptors::Binance(point) => {
                        let p = point.level();
                        let exchange = p.0;
                        (
                            p.1
                                .iter()
                                .map( |x| {
                                    Level {
                                        exchange: exchange.clone(),
                                        price: x.left,
                                        amount: x.right
                                    } as Level
                                })
                                .collect::<Vec<Level>>(),
                            p.2
                                .iter()
                                .map( |x| {
                                    Level {
                                        exchange: exchange.clone(),
                                        price: x.left,
                                        amount: x.right
                                    } as Level
                                })
                                .collect::<Vec<Level>>(),
                        )
                            // Data is an order book state snapshot (oracle) within
                            // a time frame, so we can treat it as an iterator
                            // and apply business logic within this time frame.
                    }
                    Interceptors::Bitstamp(point) => {
                        let p = point.level();
                        let exchange = p.0;
                        (
                            p.1
                                .iter()
                                .map( |x| {
                                    Level {
                                        exchange: exchange.clone(),
                                        price: x.left,
                                        amount: x.right
                                    } as Level
                                })
                                .collect::<Vec<Level>>(),
                            p.2
                                .iter()
                                .map( |x| {
                                    Level {
                                        exchange: exchange.clone(),
                                        price: x.left,
                                        amount: x.right
                                    } as Level
                                })
                                .collect::<Vec<Level>>(),
                        )
                            // we can collect at the end of the business logic,
                            // if the time buffer is enough for the process.
                    }
                    _Depth => {
                        (
                            Vec::from([Level { 
                                exchange: String::from("mock/bench"),
                                price: 0.0,
                                amount: 0.0
                            }]),
                            Vec::from([Level { 
                                exchange: String::from("mock/bench"),
                                price: 0.0,
                                amount: 0.0
                            }])
                        )
                    }
                };
                conn_lock.state.push_front(Summary {
                    spread: 0.0,
                    bids: update_point.0,
                    asks: update_point.1
                });
                let mut history = if conn_lock.state.len() > 2 {
                    conn_lock.state.iter().fold(Summary::default(), |mut acc, x| {
                        acc.bids.extend(x.bids.clone());
                        acc.asks.extend(x.asks.clone());
                        acc
                    })
                } else {
                    Summary::default()
                };
                let _cumulative_state = if conn_lock.state.len() > 3 {
                    conn_lock.state.pop_back().expect("Couldn't access history.").clone()
                } else if conn_lock.state.len() > 0 {
                    conn_lock.state.back().expect("Couldn't access history.").clone()
                } else {
                    Summary::default()
                };
                          
                history.bids.sort();
                history.asks.sort();
                let new_bids: Vec<Level> = history.bids.into_iter().rev().take(10).collect();
                let new_asks: Vec<Level> = history.asks.into_iter().take(10).collect();
                let spread = if new_bids.len() > 0 && new_asks.len() > 0 {
                    new_asks[0].price - new_bids[0].price 
                } else {
                    0.0
                };
                let new_state = Summary {
                    spread: spread,
                    bids: new_bids ,
                    asks: new_asks
                };
            
                // println!("New State: {:?}", new_state);
                conn_lock
                    .write
                    .send(ConnectionHandlerOutEvent::ConnectionEvent(Ok(
                        new_state
                    )))
                    .await?;
                conn_lock
                    .client_buf()
                    .await
                    .expect("Unable to buffer gRPC channel.");
            }

        }
        Poll::Pending => {}
    };
    println!("Connected ?");
    // grpc_pool.connect().await;

    // grpc_pool.intercept_stream().await;
    // let pool_config2 = PoolConfig {
    //     task_event_buffer_size: 1
    // };
    // let mut grpc_pool: Pool<GrpcMiddleware<Interceptors>, Interceptors> = Pool::new(1_usize, pool_config2, PoolConnectionLimits::default());

    // grpc_pool.collect_streams(Box::pin(ws_pool.local_streams));


    Ok(())
    
}