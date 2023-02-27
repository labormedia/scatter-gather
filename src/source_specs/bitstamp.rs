use super::{
    Depth,
    helpers,
    Interceptor,
    Level
};
use scatter_gather_core::connection::{ConnectionHandler, self, ConnectionHandlerOutEvent};
use serde_json;
use serde::{
    Serialize,
    Deserialize,
};
use tungstenite::Message;
use std::task::Poll;

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



#[derive(Debug)]
enum CustomDepthInEvent {
    Message(String),
    Error(Box<dyn std::error::Error + Send>)
}

impl BitstampDepthInterceptor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Depth<Level> for BitstampDepthInterceptor {
    fn helper(&self, input: String) -> Self {
        println!("Input: {:?}", input);
        match serde_json::from_str(&input){
            Ok(a) => {
                println!("Input: {:?}", a);
                a
            },
            Err(e) => {
                println!("Dropping failed parsing: {:?}", e);
                Self::default()
            }
        }
    }
    fn get_bids(&self) -> &Vec<Level> {
        &self.data.bids
    }

    fn get_asks(&self) -> &Vec<Level> {
        &self.data.asks
    }
}

impl Interceptor for BitstampDepthInterceptor {
    type Input = String;
    type Output = BitstampDepthInterceptor;

    fn intercept(&mut self, input: Self::Input) -> BitstampDepthInterceptor {
        let a = Self::helper(&self, input);
        a
    }
}

impl ConnectionHandler for BitstampDepthInterceptor {
    type InEvent = connection::ConnectionHandlerInEvent;
    type OutEvent = connection::ConnectionHandlerOutEvent<Message>;

    fn poll(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<
            connection::ConnectionHandlerOutEvent<Self::OutEvent>
        > {
        Poll::Ready(ConnectionHandlerOutEvent::ConnectionClosed(ConnectionHandlerOutEvent::ConnectionClosed(Message::Text("hello".to_string()))))
    }
    fn inject_event(&mut self, event: Self::InEvent) {
        
    }
}