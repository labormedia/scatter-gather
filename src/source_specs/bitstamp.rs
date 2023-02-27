use super::{
    Depth,
    helpers,
    Interceptor
};
use serde_json;
use serde::{
    Serialize,
    Deserialize,
};
use std::error::Error;

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

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Level {
    #[serde(deserialize_with = "helpers::quantity_from_str")]
    left: f32,
    #[serde(deserialize_with = "helpers::quantity_from_str")]
    right: f32
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
    fn helper(input: String) -> Self {
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
        let a = Self::helper(input);
        a
    }
}