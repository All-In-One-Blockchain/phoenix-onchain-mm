use anchor_lang::prelude::*;

#[account(zero_copy)]
pub struct PhoenixStrategyState {
    /// 交易者的钱包地址
    pub trader: Pubkey,
    /// 市场地址
    pub market: Pubkey,
    // Order parameters
    /// 买单序列号
    pub bid_order_sequence_number: u64,
    /// 以 ticks 为单位的买单价格
    pub bid_price_in_ticks: u64,
    /// 以 base lots 为单位的买单初始大小
    pub initial_bid_size_in_base_lots: u64,
    /// 卖单序列号
    pub ask_order_sequence_number: u64,
    /// 以 ticks 为单位的卖单价格
    pub ask_price_in_ticks: u64,
    /// 以 base lots 为单位的卖单初始大小
    pub initial_ask_size_in_base_lots: u64,
    /// 最后更新的槽号
    pub last_update_slot: u64,
    /// 最后更新的 Unix 时间戳
    pub last_update_unix_timestamp: i64,
    // Strategy parameters
    /// 报价价格和公平价格之间的基点数
    pub quote_edge_in_bps: u64,
    /// 订单名义大小以报价原子为单位
    pub quote_size_in_quote_atoms: u64,
    /// 如果设置为 true，则订单永远不会越过价差
    pub post_only: bool,
    /// 确定如何改进 BBO
    pub price_improvement_behavior: u8,
    /// 填充字段，用于对齐结构体
    pub padding: [u8; 6],
}
