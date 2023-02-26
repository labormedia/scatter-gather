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

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BinanceDepthInterceptor {
    e: String,
    E: u32,
    s: String,
    U: u32,
    u: u32,
    b: Vec<Vec<String>>,
    a: Vec<Vec<String>>
}

impl BinanceDepthInterceptor {
    pub fn new() -> Self {
        Self::default()
    }
    fn helper(input: String) -> Self {
        serde_json::from_str(&input).unwrap()
    }
}

// #[derive(Debug)]
// enum CustomDepthInEvent {
//     Message(String),
//     Error(Box<dyn std::error::Error + Send>)
// }

impl Interceptor for BinanceDepthInterceptor {
    type Input = String;
    type Output = BinanceDepthInterceptor;

    fn intercept(&mut self, input: Self::Input) -> BinanceDepthInterceptor {
        Self::helper(input)
    }
}