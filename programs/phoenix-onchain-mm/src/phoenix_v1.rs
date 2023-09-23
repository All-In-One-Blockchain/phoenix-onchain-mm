use anchor_lang::{
    __private::bytemuck::{self},
    prelude::*,
    solana_program::program::get_return_data,
};
use phoenix::program::MarketHeader;
use phoenix::{
    quantities::WrapperU64,
    state::{
        markets::{FIFOOrderId, FIFORestingOrder, Market},
        OrderPacket, Side,
    },
};

use crate::errors::StrategyError;

pub const PHOENIX_MARKET_DISCRIMINANT: u64 = 8167313896524341111;

#[derive(Clone)]
pub struct PhoenixV1;

impl anchor_lang::Id for PhoenixV1 {
    fn id() -> Pubkey {
        phoenix::id()
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
struct DeserializedFIFOOrderId {
    pub price_in_ticks: u64,
    pub order_sequence_number: u64,
}

pub fn parse_order_ids_from_return_data(order_ids: &mut Vec<FIFOOrderId>) -> Result<()> {
    if let Some((program_id, orders_data)) = get_return_data() {
        msg!("Found return data");
        if program_id == phoenix::id() && !orders_data.is_empty() {
            msg!("Found orders in return data");
            Vec::<DeserializedFIFOOrderId>::try_from_slice(&orders_data)?
                .into_iter()
                .for_each(|o| {
                    order_ids.push(FIFOOrderId::new_from_untyped(
                        o.price_in_ticks,
                        o.order_sequence_number,
                    ))
                });
        } else {
            msg!("No orders in return data");
        }
    }
    Ok(())
}

pub fn load_header(info: &AccountInfo) -> Result<MarketHeader> {
    require!(
        info.owner == &phoenix::id(),
        StrategyError::InvalidPhoenixProgram
    );
    let data = info.data.borrow();
    let header =
        bytemuck::try_from_bytes::<MarketHeader>(&data[..std::mem::size_of::<MarketHeader>()])
            .map_err(|_| {
                msg!("Failed to parse Phoenix market header");
                StrategyError::FailedToDeserializePhoenixMarket
            })?;
    require!(
        header.discriminant == PHOENIX_MARKET_DISCRIMINANT,
        StrategyError::InvalidPhoenixProgram,
    );
    Ok(*header)
}

pub fn get_best_bid_and_ask(
    market: &dyn Market<Pubkey, FIFOOrderId, FIFORestingOrder, OrderPacket>,
    trader_index: u64,
) -> (u64, u64) {
    let best_bid = market
        .get_book(Side::Bid)
        .iter()
        .find(|(_, o)| o.trader_index != trader_index)
        .map(|(o, _)| o.price_in_ticks.as_u64())
        .unwrap_or_else(|| 1);
    let best_ask = market
        .get_book(Side::Ask)
        .iter()
        .find(|(_, o)| o.trader_index != trader_index)
        .map(|(o, _)| o.price_in_ticks.as_u64())
        .unwrap_or_else(|| u64::MAX);
    (best_bid, best_ask)
}

pub fn get_bid_price_in_ticks(
    fair_price_in_quote_atoms_per_raw_base_unit: u64,
    header: &MarketHeader,
    edge_in_bps: u64,
) -> u64 {
    let (fair_price_in_ticks, edge_in_ticks) = common_for_bid_and_ask_price_in_ticks(
        fair_price_in_quote_atoms_per_raw_base_unit,
        header,
        edge_in_bps,
    );
    fair_price_in_ticks - edge_in_ticks
}

pub fn get_ask_price_in_ticks(
    fair_price_in_quote_atoms_per_raw_base_unit: u64,
    header: &MarketHeader,
    edge_in_bps: u64,
) -> u64 {
    let (fair_price_in_ticks, edge_in_ticks) = common_for_bid_and_ask_price_in_ticks(
        fair_price_in_quote_atoms_per_raw_base_unit,
        header,
        edge_in_bps,
    );
    fair_price_in_ticks + edge_in_ticks
}

fn common_for_bid_and_ask_price_in_ticks(
    fair_price_in_quote_atoms_per_raw_base_unit: u64,
    header: &MarketHeader,
    edge_in_bps: u64,
) -> (u64, u64) {
    let fair_price_in_ticks = fair_price_in_quote_atoms_per_raw_base_unit
        * header.raw_base_units_per_base_unit as u64
        / header.get_tick_size_in_quote_atoms_per_base_unit().as_u64();
    let edge_in_ticks = edge_in_bps * fair_price_in_ticks / 10_000;
    (fair_price_in_ticks, edge_in_ticks)
}
