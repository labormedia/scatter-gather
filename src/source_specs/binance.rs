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
    helpers
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json;

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

impl BinanceDepthInterceptor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Depth<Level> for BinanceDepthInterceptor {
    fn helper(input: String) -> Self {
        println!("Input: {:?}", input);
        serde_json::from_str(&input).expect("Parsing error.")
    }
    fn get_bids(&self) -> &Vec<Level> {
        &self.b
    }

    fn get_asks(&self) -> &Vec<Level> {
        &self.a
    }
}

impl Interceptor for BinanceDepthInterceptor {
    type Input = String;
    type Output = BinanceDepthInterceptor;

    fn intercept(&mut self, input: Self::Input) -> BinanceDepthInterceptor {
        Self::helper(input)
    }
}