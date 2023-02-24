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



use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BinanceDepth {
    e: String,
    E: u32,
    s: String,
    U: u32,
    u: u32,
    b: Vec<Vec<String>>,
    a: Vec<Vec<String>>
}
