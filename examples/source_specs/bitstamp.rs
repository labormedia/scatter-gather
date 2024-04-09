use super::{
    Depth,
    Interceptor,
    Level
};
use scatter_gather_core::connection::{ConnectionHandler, self};
use serde_json;
use serde::{
    Serialize,
    Deserialize,
};
use tungstenite::Message;
use core::task::{
    Poll,
    Context
};

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BitstampDepthInterceptor {
    event: String,
    channel: String,
    data: Data
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Data {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<Level>,
    asks: Vec<Level>,
}

impl BitstampDepthInterceptor {
    pub fn new() -> Self {
        Self::default()
    }
    
}

impl Depth<Level> for BitstampDepthInterceptor {
    fn level(self) -> (String, Vec<Level>, Vec<Level>) {
        let mut a = Vec::new();
        a.extend_from_slice(&self.data.asks);
        let mut b = Vec::new();
        b.extend_from_slice(&self.data.bids);

        (self.exchange(), b, a)
    }
    fn exchange(self) -> String { String::from("Bitstamp") }
    fn get_bids(self) -> Vec<Level> {
        self.data.bids
    }

    fn get_asks(self) -> Vec<Level> {
        self.data.asks
    }
}

impl Interceptor for BitstampDepthInterceptor {
    type Input = String;
    type Output = BitstampDepthInterceptor;
    fn helper(input: String) -> Self {
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
    fn intercept(input: Self::Input) -> BitstampDepthInterceptor {
        Self::helper(input)
    }
}

// impl ConnectionHandler<'_> for BitstampDepthInterceptor {
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
//     fn inject_event(&mut self, _event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
//         Ok(())
//     }
//     fn eject_event(&mut self, _event: Self::OutEvent) -> Result<(), Box<dyn std::error::Error>> {
//         Ok(())
//     }
//     fn as_any(&self) -> &dyn std::any::Any {
//         self as _
//     }
// }

