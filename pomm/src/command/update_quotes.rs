use crate::config::PhoenixOnChainMMConfig;
use crate::constant::BASE;
use crate::constant::{PHOENIX_ONCHAIN_MM_ORACLE_SEED, PHOENIX_ONCHAIN_MM_STRATEGY_SEED};
use crate::errors::Error;
use crate::ids;
use crate::utils::get_pomm_config;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use phoenix::program::get_seat_address;
use phoenix::program::get_vault_address;
use phoenix::program::MarketHeader;
use phoenix_onchain_mm::accounts::UpdateQuotes as UpdateQuotesAccounts;
use phoenix_onchain_mm::instruction::UpdateQuotes as UpdateQuotesInstruction;
use phoenix_onchain_mm::oracle::OracleConfig;
use phoenix_onchain_mm::OrderParams;
use phoenix_onchain_mm::PriceImprovementBehavior;
use phoenix_onchain_mm::StrategyParams;
use phoenix_sdk::sdk_client::SDKClient;
use pyth_sdk_solana::load_price_feed_from_account;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::Instruction;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateQuotes {
    #[structopt(long, default_value = "60")]
    pub rebalance_sec: u64,
}

impl UpdateQuotes {
    // TODO: It should automatically rebalance and be able to maintain 24/7 liquidity.
    pub async fn run(&self) -> anyhow::Result<()> {
        // 创建一个异步任务线程执行定时器任务
        let update_quote_task_handle = tokio::spawn(update_quote());

        // 创建一个异步任务线程执行另一个任务
        let rebalance_task_handle = tokio::spawn(rebalance_task(self.rebalance_sec));

        // 等待异步任务执行完成
        let (_v1, _v2) = tokio::join!(update_quote_task_handle, rebalance_task_handle);

        Ok(())
    }
}

async fn rebalance_task(reblance_sec: u64) -> anyhow::Result<()> {
    let phoneix_config = get_pomm_config().map_err(|e| Error::from(e.to_string()))?;

    let (commitment, payer, rpc_enpoint) = phoneix_config
        .read_global_config()
        .map_err(|e| Error::from(e.to_string()))?;

    let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

    let sdk = SDKClient::new(&payer, &rpc_enpoint)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let PhoenixOnChainMMConfig { market, .. } = phoneix_config.phoenix.clone();

    let base_account = phoneix_config
        .phoenix
        .get_base_oracle_account()
        .map_err(|e| Error::from(e.to_string()))?;
    let quote_account = phoneix_config
        .phoenix
        .get_quote_oracle_account()
        .map_err(|e| Error::from(e.to_string()))?;

    // get price data from key
    let mut base_price_account = client
        .get_account(&base_account)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let base_price_feed = load_price_feed_from_account(&base_account, &mut base_price_account)
        .map_err(|e| Error::from(e.to_string()))?;

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    let base_price = base_price_feed
        .get_price_no_older_than(current_time, 60)
        .ok_or(anyhow::anyhow!("base price is unavaiable"))?;

    let real_base_price = base_price.price as f64 * BASE.powi(base_price.expo);

    println!(
        "Base price ........... {} x 10^{} = {}",
        base_price.price, base_price.expo, real_base_price
    );

    // get price data from key
    let mut quote_price_account = client
        .get_account(&quote_account)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let quote_price_feed = load_price_feed_from_account(&quote_account, &mut quote_price_account)
        .map_err(|e| Error::from(e.to_string()))?;

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::from(e.to_string()))?
        .as_secs() as i64;

    let quote_price = quote_price_feed
        .get_price_no_older_than(current_time, 60)
        .ok_or(anyhow::anyhow!("base price is unavaiable"))?;

    let real_quote_price = quote_price.price as f64 * BASE.powi(quote_price.expo);

    println!(
        "Quote price ........... {} x 10^{} = {}",
        quote_price.price, quote_price.expo, real_quote_price
    );

    let data = client
        .get_account_data(&phoneix_config.phoenix.market)
        .await?;
    let header =
        bytemuck::try_from_bytes::<MarketHeader>(&data[..std::mem::size_of::<MarketHeader>()])
            .map_err(|_| anyhow::Error::msg("Failed to parse Phoenix market header"))?;

    let quote_token_account =
        get_associated_token_address(&payer.pubkey(), &header.quote_params.mint_key);
    let base_token_account =
        get_associated_token_address(&payer.pubkey(), &header.base_params.mint_key);

    let quote_start_balance = client
        .get_token_account_balance(&quote_token_account)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ui_amount_string;

    let base_start_balance = client
        .get_token_account_balance(&base_token_account)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ui_amount_string;

    println!(
        "Base Balance: {:#?}, QuoteBalance: {:#?}\n",
        base_start_balance, quote_start_balance
    );

    let base_balance = real_base_price
        * base_start_balance
            .parse::<f64>()
            .map_err(|e| Error::from(e.to_string()))?;
    let quote_balance = real_quote_price
        * quote_start_balance
            .parse::<f64>()
            .map_err(|e| Error::from(e.to_string()))?;
    let target_balance_ratio = 1.0;
    loop {
        rebalance(
            &market,
            base_balance,
            real_base_price,
            quote_balance,
            real_quote_price,
            target_balance_ratio,
            &sdk,
        )
        .await?;

        tokio::time::sleep(std::time::Duration::from_secs(reblance_sec)).await;
    }
}

async fn rebalance(
    market_key: &Pubkey,
    base_balance: f64,
    base_price: f64,
    quote_balance: f64,
    quote_price: f64,
    target_balance_ratio: f64,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    let current_ratio = base_balance / quote_balance;

    if current_ratio > target_balance_ratio + 0.05 {
        let balance_to_convert = (current_ratio - target_balance_ratio) * base_balance;
        let base_size = (base_balance / base_price) as u64;
        let (sig, _fills) = sdk
            .send_ioc(
                market_key,
                sdk.float_price_to_ticks_rounded_down(market_key, balance_to_convert)
                    .map_err(|e| Error::from(e.to_string()))?,
                phoenix::state::Side::Ask,
                base_size,
            )
            .await
            .ok_or(anyhow::anyhow!("send ioc retuen error"))?;

        println!(
            "Rebalance Base Coin : https://explorer.solana.com/tx/{}?cluster=devnet",
            sig
        );
    } else if current_ratio < target_balance_ratio - 0.05 {
        let balance_to_convert = (target_balance_ratio - current_ratio) * quote_balance;
        let quote_size = (base_balance / quote_price) as u64;
        let (sig, _fills) = sdk
            .send_ioc(
                market_key,
                sdk.float_price_to_ticks_rounded_down(market_key, balance_to_convert)
                    .map_err(|e| Error::from(e.to_string()))?,
                phoenix::state::Side::Bid,
                quote_size,
            )
            .await
            .ok_or(anyhow::anyhow!("send ioc retuen error"))?;
        println!(
            "Rebalance Quote Coin : https://explorer.solana.com/tx/{}?cluster=devnet",
            sig
        );
    }

    Ok(())
}

async fn update_quote() -> anyhow::Result<()> {
    let phoneix_config = get_pomm_config().map_err(|e| Error::from(e.to_string()))?;

    let (commitment, payer, rpc_enpoint) = phoneix_config
        .read_global_config()
        .map_err(|e| Error::from(e.to_string()))?;

    let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

    let mut sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let PhoenixOnChainMMConfig {
        market,
        ticker: _,
        quote_edge_in_bps,
        quote_size,
        quote_refresh_frequency_in_ms,
        price_improvement_behavior,
        post_only,
    } = phoneix_config.phoenix.clone();

    let oracle_base_account = phoneix_config
        .phoenix
        .get_base_oracle_account()
        .map_err(|e| Error::from(e.to_string()))?;
    let oracle_quote_account = phoneix_config
        .phoenix
        .get_quote_oracle_account()
        .map_err(|e| Error::from(e.to_string()))?;

    // add market pubkey to sdk
    sdk.add_market(&market).await?;

    let maker_setup_instructions = sdk
        .get_maker_setup_instructions_for_market(&market)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let ix = sdk
        .client
        .sign_send_instructions(maker_setup_instructions, vec![])
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    println!(
        "Claim maker seta: https://explorer.solana.com/tx/{}?cluster=devnet",
        ix
    );

    let (strategy_key, _bump_seed) = Pubkey::find_program_address(
        &[
            PHOENIX_ONCHAIN_MM_STRATEGY_SEED,
            payer.pubkey().as_ref(),
            market.as_ref(),
        ],
        &ids::phoenix_onchain_mm_program::id(),
    );

    let (oracle_account, _) = Pubkey::find_program_address(
        &[
            PHOENIX_ONCHAIN_MM_ORACLE_SEED,
            payer.pubkey().as_ref(),
            market.as_ref(),
        ],
        &ids::phoenix_onchain_mm_program::id(),
    );

    let price_improvement = match price_improvement_behavior.as_str() {
        "Join" | "join" => PriceImprovementBehavior::Join,
        "Dime" | "dime" => PriceImprovementBehavior::Dime,
        "Ignore" | "ignore" => PriceImprovementBehavior::Ignore,
        _ => PriceImprovementBehavior::Join,
    };

    let params = StrategyParams {
        quote_edge_in_bps: Some(quote_edge_in_bps),
        quote_size_in_quote_atoms: Some(quote_size),
        price_improvement_behavior: Some(price_improvement),
        post_only: Some(post_only),
        oracle_account_config: OracleConfig {
            oracle_base_account,
            oracle_quote_account,
        },
    };

    let data = client
        .get_account_data(&market)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let header =
        bytemuck::try_from_bytes::<MarketHeader>(&data[..std::mem::size_of::<MarketHeader>()])
            .map_err(|_| anyhow::Error::msg("Failed to parse Phoenix market header"))?;

    println!("Quote Params: {:#?}", params);

    loop {
        let args = UpdateQuotesInstruction {
            params: OrderParams {
                strategy_params: params,
            },
        };

        let quote_account =
            get_associated_token_address(&payer.pubkey(), &header.quote_params.mint_key);

        let base_account =
            get_associated_token_address(&payer.pubkey(), &header.base_params.mint_key);

        let accounts = UpdateQuotesAccounts {
            phoenix_strategy: strategy_key,
            oracle_account,
            oracle_base_price: oracle_base_account,
            oracle_quote_price: oracle_quote_account,
            market,
            user: payer.pubkey(),
            phoenix_program: phoenix::id(),
            log_authority: phoenix::phoenix_log_authority::id(),
            seat: get_seat_address(&market, &payer.pubkey()).0,
            quote_account,
            base_account,
            quote_vault: get_vault_address(&market, &header.quote_params.mint_key).0,
            base_vault: get_vault_address(&market, &header.base_params.mint_key).0,
            token_program: spl_token::id(),
        };

        let ix = Instruction {
            program_id: ids::phoenix_onchain_mm_program::id(),
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            client
                .get_latest_blockhash()
                .await
                .map_err(|e| Error::from(e.to_string()))?,
        );

        match client
            .send_and_confirm_transaction(&transaction)
            .await
            .map(|sig| {
                println!(
                    "Updating quotes: https://explorer.solana.com/tx/{}?cluster=devnet",
                    sig
                );
            }) {
            Ok(_) => {}
            Err(e) => println!("Failed to update quotes: {:#?}", e),
        }

        tokio::time::sleep(std::time::Duration::from_millis(
            quote_refresh_frequency_in_ms,
        ))
        .await;
    }
}
