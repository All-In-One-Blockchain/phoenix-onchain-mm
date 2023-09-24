use crate::config::{Config as PhoenixConfig, PhoenixOnChainMMConfig};
use crate::utils::get_pomm_config;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use phoenix::program::get_seat_address;
use phoenix::program::get_vault_address;
use phoenix::program::MarketHeader;
use phoenix_onchain_mm::accounts::UpdateQuotes as UpdateQuotesAccounts;
use phoenix_onchain_mm::instruction::UpdateQuotes as UpdateQuotesInstruction;
use phoenix_onchain_mm::OrderParams;
use phoenix_onchain_mm::PriceImprovementBehavior;
use phoenix_onchain_mm::StrategyParams;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateQuotes {}

impl UpdateQuotes {
    // TODO: It should automatically rebalance and be able to maintain 24/7 liquidity.
    pub async fn run(&self) -> anyhow::Result<()> {
        // 创建一个异步任务线程执行定时器任务
        let update_quote_task_handle = tokio::spawn(update_quote());

        // 创建一个异步任务线程执行另一个任务
        let time_task_handle = tokio::spawn(time_task());

        // 等待异步任务执行完成
        let (_v1, _v2) = tokio::join!(update_quote_task_handle, time_task_handle);

        Ok(())
    }
}

async fn time_task() -> anyhow::Result<()> {
    let phoneix_config = get_pomm_config()?;

    let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;
    let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

    let mut sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

    let PhoenixOnChainMMConfig { market, .. } = phoneix_config.phoenix;

    let data = client
        .get_account_data(&phoneix_config.phoenix.market)
        .await?;
    let header =
        bytemuck::try_from_bytes::<MarketHeader>(&data[..std::mem::size_of::<MarketHeader>()])
            .map_err(|_| anyhow::Error::msg("Failed to parse Phoenix market header"))?;

    let _base_decimals = u64::pow(10, header.base_params.decimals);
    let _quote_decimals = u64::pow(10, header.quote_params.decimals);

    loop {
        // 模拟一秒钟执行一次定时器任务
        //

        // todo 这里需要获取 base 与 quote 的余额, 以及当前的价格
        // 如果base token的价值和quote的价格打破了 50% 的比例, 那么就需要进行调整
        // let base_balance = sdk.get_token_balance(&market.base_mint).await?;
        // let quote_balance = sdk.get_token_balance(&market.quote_mint).await?;
        // let price = sdk.get_price(&market).await?;
        // let base_value = base_balance * price;
        // let quote_value = quote_balance;
        // let base_quote_ratio = base_value / quote_value;
        // if base_quote_ratio > 1.5 || base_quote_ratio < 0.5 {
        //    // todo 这里需要调整 base 与 quote 的余额
        //  }
        //

        let quote_token_account =
            get_associated_token_address(&payer.pubkey(), &header.quote_params.mint_key);
        let base_token_account =
            get_associated_token_address(&payer.pubkey(), &header.base_params.mint_key);

        let quote_start_balance = client
            .get_token_account_balance(&quote_token_account)
            .await?
            .ui_amount_string;

        let base_start_balance = client
            .get_token_account_balance(&base_token_account)
            .await?
            .ui_amount_string;

        // let (_sig, fills) = sdk
        //     .send_ioc(&market, price_in_ticks * tick_size, side, size_in_base_lots)
        //     .await
        //     .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(60 * 60 * 2)).await;
        println!("定时器任务执行中...");
    }
}

async fn update_quote() -> anyhow::Result<()> {
    let phoneix_config = get_pomm_config()?;

    let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;
    let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

    let mut sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

    let PhoenixOnChainMMConfig {
        market,
        ticker: _,
        quote_edge_in_bps,
        quote_size,
        quote_refresh_frequency_in_ms,
        price_improvement_behavior,
        post_only,
    } = phoneix_config.phoenix;

    // add market pubkey to sdk
    sdk.add_market(&market).await?;

    let maker_setup_instructions = sdk.get_maker_setup_instructions_for_market(&market).await?;
    sdk.client
        .sign_send_instructions(maker_setup_instructions, vec![])
        .await
        .unwrap();

    let (strategy_key, _bump_seed) = Pubkey::find_program_address(
        &[b"phoenix", payer.pubkey().as_ref(), market.as_ref()],
        &phoenix_onchain_mm::id(),
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
    };

    let data = client.get_account_data(&market).await?;
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

        let accounts = UpdateQuotesAccounts {
            phoenix_strategy: strategy_key,
            market,
            user: payer.pubkey(),
            phoenix_program: phoenix::id(),
            log_authority: phoenix::phoenix_log_authority::id(),
            seat: get_seat_address(&market, &payer.pubkey()).0,
            quote_account: get_associated_token_address(
                &payer.pubkey(),
                &header.quote_params.mint_key,
            ),
            base_account: get_associated_token_address(
                &payer.pubkey(),
                &header.base_params.mint_key,
            ),
            quote_vault: get_vault_address(&market, &header.quote_params.mint_key).0,
            base_vault: get_vault_address(&market, &header.base_params.mint_key).0,
            token_program: spl_token::id(),
        };

        let ix = Instruction {
            program_id: phoenix_onchain_mm::id(),
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            client.get_latest_blockhash().await?,
        );
        match client
            .send_and_confirm_transaction(&transaction)
            .await
            .map(|sig| {
                println!("Updating quotes: {}", sig);
            }) {
            Ok(_) => {}
            Err(e) => println!("Failed to update quotes: {}", e),
        }

        tokio::time::sleep(std::time::Duration::from_millis(
            quote_refresh_frequency_in_ms,
        ))
        .await;
    }
}
