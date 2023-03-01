use super::{
    Depth,
    Interceptor,
    Level
};
use scatter_gather_core::connection::{ConnectionHandler, self, ConnectionHandlerOutEvent, ConnectionHandlerInEvent};
use serde_json;
use serde::{
    Serialize,
    Deserialize,
};
use futures::Future;
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
}

impl Depth<Level> for BitstampDepthInterceptor {
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

impl connection::ConnectionHandler for BitstampDepthInterceptor {
    type InEvent = connection::ConnectionHandlerInEvent<Message>;
    type OutEvent = connection::ConnectionHandlerOutEvent<Message>;

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::OutEvent>
    {
        // Poll::Ready(connection::ConnectionHandlerOutEvent::ConnectionClosed(Message::Text("hello".to_string())))
        Poll::Pending
    }
    fn inject_event(&mut self, event: Self::InEvent) {
        
    }
    fn eject_event(&mut self, event: Self::OutEvent) {
        
    }
}

impl Future for BitstampDepthInterceptor {
    type Output = &'static dyn Depth<Level>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inject_event(ConnectionHandlerInEvent::Connect);
        Poll::Pending
    }
}