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

use super::*;
use serde_json;
use serde::{
    de,
    Deserializer
};

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BinanceDepthInterceptor {
    e: String,
    E: i64,
    s: String,
    U: u32,
    u: u32,
    b: Vec<Level>,
    a: Vec<Level>
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Level{
    #[serde(deserialize_with = "BinanceDepthInterceptor::quantity_from_str")]
    left: f32,
    #[serde(deserialize_with = "BinanceDepthInterceptor::quantity_from_str")]
    right: f32
}

#[derive(Debug)]
enum CustomDepthInEvent {
    Message(String),
    Error(Box<dyn std::error::Error + Send>)
}

impl BinanceDepthInterceptor {
    pub fn new() -> Self {
        Self::default()
    }
    fn helper(input: String) -> Self {
        println!("Input: {:?}", input);
        serde_json::from_str(&input).expect("Parsing error.")
    }
    pub fn get_bids(&self) -> &Vec<Level> {
        &self.b
    }

    pub fn get_asks(&self) -> &Vec<Level> {
        &self.a
    }

    fn quantity_from_str<'a, D>(input: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'a>,
    {
        let str_val = String::deserialize(input)?;
        str_val.parse::<f32>().map_err(de::Error::custom)
    }
}

impl Interceptor for BinanceDepthInterceptor {
    type Input = String;
    type Output = BinanceDepthInterceptor;

    fn intercept(&mut self, input: Self::Input) -> BinanceDepthInterceptor {
        Self::helper(input)
    }
}