use std::collections::HashMap;
use crate::photon_decode::PhotonValue;
use crate::constants::*;

pub fn route_reliable_message(parameters: &HashMap<u8, PhotonValue>) {
    // 1. Check for Event Code (Key 252)
    if let Some(code_val) = parameters.get(&252) {
        let code = match code_val {
            PhotonValue::Int8(c) => *c as u8,
            PhotonValue::Int16(c) => *c as u8,
            PhotonValue::Int32(c) => *c as u8,
            _ => 0,
        };

        match code {
            OP_EVENT_UPDATE_SILVER => {
                if let Some(PhotonValue::Int64(silver)) = parameters.get(&1) {
                    println!("💰 [Player] Silver Balance Updated: {} silver", silver / 10000);
                }
            },
            OP_LOCATION_CHANGED => {
                if let Some(PhotonValue::Int32(zone_id)) = parameters.get(&0) {
                    println!("🌍 [World] Location Changed! Zone ID: {}", zone_id);
                }
            },
            EVENT_NEW_ITEM => {
                println!("🎒 [Inventory] New item detected in inventory or container.");
            },
            _ => {} // Other events
        }
    }

    // 2. Check for Operation Code (Key 253)
    if let Some(code_val) = parameters.get(&253) {
        let code = match code_val {
            PhotonValue::Int8(c) => *c as u8,
            PhotonValue::Int16(c) => *c as u8,
            PhotonValue::Int32(c) => *c as u8,
            _ => 0,
        };

        match code {
            OP_AUCTION_GET_REQUESTS => {
                println!("⚖️ [Market] Intercepted Buy Orders (Requests) Update!");
                handle_market_orders(parameters);
            },
            OP_AUCTION_GET_OFFERS => {
                println!("⚖️ [Market] Intercepted Sell Orders (Offers) Update!");
                handle_market_orders(parameters);
            },
            OP_AUCTION_GET_MY_ORDERS => {
                println!("💼 [Market] Intercepted 'My Orders' Page!");
            },
            OP_AUCTION_AVG_GET => {
                println!("📈 [Market] Intercepted Historical Average Price Data!");
            },
            _ => {} // Other operations
        }
    }
}

fn handle_market_orders(parameters: &HashMap<u8, PhotonValue>) {
    // Albion transfers the array of order JSON strings under parameter key 0
    if let Some(PhotonValue::Slice(orders_slice)) = parameters.get(&0) {
        println!("📊 Parsing {} market entries...", orders_slice.len());
        
        for value in orders_slice {
            if let PhotonValue::String(json_str) = value {
                println!("   -> Entry: {}", json_str);
            }
        }
        println!("--------------------------------------------------");
    } else {
        println!("[Market] Alert intercepted, but Parameter 0 (data slice) was missing.");
    }
}