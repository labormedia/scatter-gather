// Binance Depth Payload Example:
// {
//     "e": "depthUpdate", // Event type
//     "E": 1672515782136, // Event time
//     "s": "BNBBTC",      // Symbol
//     "U": 157,           // First update ID in event
//     "u": 160,           // Final update ID in event
//     "b": [              // Bids to be updated
//       [
//         "0.0024",       // Price level to be updated
//         "10"            // Quantity
//       ]
//     ],
//     "a": [              // Asks to be updated
//       [
//         "0.0026",       // Price level to be updated
//         "100"           // Quantity
//       ]
//     ]
//   }
use super::{
    Depth,
    Interceptor,
    Level
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json;
use futures::Future;

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BinanceDepthInterceptor {
    e: String,
    E: i64,
    s: String,
    U: i64,
    u: i64,
    b: Vec<Level>,
    a: Vec<Level>
}
use scatter_gather_core::connection::{self, ConnectionHandler, ConnectionHandlerOutEvent, ConnectionHandlerInEvent};
use core::task::{
    Poll,
    Context
};
use tungstenite::Message;

impl BinanceDepthInterceptor {
    pub fn new() -> Self {
        Self::default()
    }
    // fn exchange() -> String { String::from("Binance") }
}

impl Depth<Level> for BinanceDepthInterceptor {
    fn level(self) -> (String, Vec<Level>, Vec<Level>) {
        let mut a = vec![];
        a.extend_from_slice(&self.a);
        let mut b = vec![];
        b.extend_from_slice(&self.b);
        

        (self.exchange(), b, a)
    }
    fn exchange(self) -> String { String::from("Binance") }
    fn get_bids(self) -> Vec<Level> {
        self.b
    }

    fn get_asks(self) -> Vec<Level> {
        self.a
    }
}

impl Interceptor for BinanceDepthInterceptor {
    type Input = String;
    type Output = BinanceDepthInterceptor;
    fn helper(input: Self::Input) -> Self::Output {
        // #[cfg(debug_assertions)]
        // println!("Input: {:?}", input);
        match serde_json::from_str(&input){
            Ok(a) => {
                // #[cfg(debug_assertions)]
                // println!("Input: {:?}", a);
                a
            },
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("Dropping failed parsing: {:?}", e);
                Self::default()
            }
        }
    }
    fn intercept(input: Self::Input) -> Self::Output {
        Self::Output::helper(input)
    }
}

// impl ConnectionHandler<'_> for BinanceDepthInterceptor {
//     type InEvent = connection::ConnectionHandlerInEvent;
//     type OutEvent = connection::ConnectionHandlerOutEvent<Message>;

//     fn poll(
//         self,
//         _cx: &mut Context<'_>,
//     ) -> Poll<Self::OutEvent> 
//     {
//         // Poll::Ready(connection::ConnectionHandlerOutEvent::ConnectionClosed(Message::Text("hello".to_string())))
//         Poll::Pending
//     }
//     fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
//         #[cfg(debug_assertions)]
//         println!("Hello Future! InEvent: {:?}", event);
//         Ok(())
//     }
//     fn eject_event(&mut self, event: Self::OutEvent) -> Result<(), Box<dyn std::error::Error>> {
//         #[cfg(debug_assertions)]
//         println!("Hello Future! OutEvent: {:?}", event);
//         Ok(())
//     }
//     fn as_any(&self) -> &dyn std::any::Any {
//         self as _
//     }
// }

// impl Future for BinanceDepthInterceptor {
//     type Output = &'static dyn Depth<Level>;

//     fn poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
//         self.inject_event(ConnectionHandlerInEvent::Connect).expect("Couldn't inject event.");
//         Poll::Pending
//     }
// }