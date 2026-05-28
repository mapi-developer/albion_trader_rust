// Operations (Requests/Responses via code 253)
pub const OP_AUCTION_GET_REQUESTS: u8 = 120; // Buy Orders tab
pub const OP_AUCTION_GET_OFFERS: u8 = 118;   // Sell Orders tab
pub const OP_AUCTION_GET_MY_ORDERS: u8 = 122;// My Orders tab
pub const OP_AUCTION_AVG_GET: u8 = 126;      // Historical price data

// Events (Server broadcasts via code 252)
pub const OP_EVENT_UPDATE_SILVER: u8 = 21;
pub const OP_LOCATION_CHANGED: u8 = 2;       // Zone change
pub const OP_JOIN_FINISHED: u8 = 1;          // Logged in / Finished loading zone
pub const EVENT_NEW_ITEM: u8 = 19;           // Items added to inventory/chests