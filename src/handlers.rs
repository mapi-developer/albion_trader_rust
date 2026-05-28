use std::collections::HashMap;
use crate::photon_decode::PhotonValue;
use crate::constants::*;

pub fn route_reliable_message(message_type: u8, code: u8, parameters: &HashMap<u8, PhotonValue>) {
    match message_type {
        // message_type 2 or 3/7 = Operation Request/Response
        2 | 3 | 7 => {
            if code == OP_AUCTION_GET_REQUESTS {
                println!("⚖️ [Market] Intercepted Buy Orders (Requests) Update!");
                handle_market_orders(parameters);
            } else if code == OP_AUCTION_GET_OFFERS {
                println!("⚖️ [Market] Intercepted Sell Orders (Offers) Update!");
                handle_market_orders(parameters);
            } else if code == OP_AUCTION_GET_MY_ORDERS {
                println!("💼 [Market] Intercepted 'My Orders' Page!");
            } else if code == OP_AUCTION_AVG_GET {
                println!("📈 [Market] Intercepted Historical Average Price Data!");
            }
        },
        // message_type 4 = Event Data
        4 => {
            if code == OP_EVENT_UPDATE_SILVER {
                if let Some(PhotonValue::Int64(silver)) = parameters.get(&1) {
                    println!("💰 [Player] Silver Balance Updated: {} silver", silver / 10000);
                }
            } else if code == OP_LOCATION_CHANGED {
                if let Some(PhotonValue::Int32(zone_id)) = parameters.get(&0) {
                    println!("🌍 [World] Location Changed! Zone ID: {}", zone_id);
                }
            } else if code == EVENT_NEW_ITEM {
                println!("🎒 [Inventory] New item detected in inventory or container.");
            }
        },
        _ => {}
    }
}

fn handle_market_orders(parameters: &HashMap<u8, PhotonValue>) {
    // Albion transfers the array of order JSON strings under parameter key 0
    if let Some(PhotonValue::Slice(orders_slice)) = parameters.get(&0) {
        println!("📊 Parsing {} market entries...", orders_slice.len());
        
        for value in orders_slice {
            if let PhotonValue::String(json_str) = value {
                // This string is a raw JSON representing a single market listing
                println!("   -> Entry: {}", json_str);
            }
        }
        println!("--------------------------------------------------");
    }
}