use anchor_lang::prelude::*;

#[account(zero_copy)]
pub struct PhoenixStrategyState {
    pub trader: Pubkey,
    pub market: Pubkey,
    // Order parameters
    pub bid_order_sequence_number: u64,
    pub bid_price_in_ticks: u64,
    pub initial_bid_size_in_base_lots: u64,
    pub ask_order_sequence_number: u64,
    pub ask_price_in_ticks: u64,
    pub initial_ask_size_in_base_lots: u64,
    pub last_update_slot: u64,
    pub last_update_unix_timestamp: i64,
    // Strategy parameters
    pub quote_edge_in_bps: u64,
    pub quote_size_in_quote_atoms: u64,
    pub post_only: bool,
    pub price_improvement_behavior: u8,
    pub padding: [u8; 6],
}
