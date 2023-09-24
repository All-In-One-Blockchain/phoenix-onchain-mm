use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use phoenix::program::get_seat_address;
use phoenix::program::get_vault_address;
use phoenix::program::MarketHeader;
use phoenix_onchain_mm::accounts::{
    Initialize as InitializeAccounts, UpdateQuotes as UpdateQuotesAccounts,
};
use phoenix_onchain_mm::instruction::{
    Initialize as InitializeInstruction, UpdateQuotes as UpdateQuotesInstruction,
};
use phoenix_onchain_mm::OrderParams;
use phoenix_onchain_mm::PriceImprovementBehavior;
use phoenix_onchain_mm::StrategyParams;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use crate::config::{Config as PhoenixConfig, PhoenixOnChainMMConfig};

pub async fn run(phoneix_config: PhoenixConfig) -> anyhow::Result<()> {
    let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;
    let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

    let sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

    let PhoenixOnChainMMConfig {
        market,
        ticker,
        quote_edge_in_bps,
        quote_size,
        quote_refresh_frequency_in_ms,
        price_improvement_behavior,
        post_only,
    } = phoneix_config.phoenix;

    let maker_setup_instructions = sdk.get_maker_setup_instructions_for_market(&market).await?;
    sdk.client
        .sign_send_instructions(maker_setup_instructions, vec![])
        .await
        .unwrap();

    let (strategy_key, _bump_seed) = Pubkey::find_program_address(
        &[b"phoenix", payer.pubkey().as_ref(), market.as_ref()],
        &phoenix_onchain_mm::id(),
    );

    let mut create = false;
    match client.get_account(&strategy_key).await {
        Ok(acc) => {
            if acc.data.is_empty() {
                create = true;
            }
        }
        Err(_) => {
            create = true;
        }
    }

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
    if create {
        let initialize_data = InitializeInstruction { params };
        let initialize_accounts = InitializeAccounts {
            phoenix_strategy: strategy_key,
            market,
            user: payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        };

        let ix = Instruction {
            program_id: phoenix_onchain_mm::id(),
            accounts: initialize_accounts.to_account_metas(None),
            data: initialize_data.data(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            client.get_latest_blockhash().await?,
        );
        let txid = client.send_and_confirm_transaction(&transaction).await?;
        println!("Creating strategy account: {}", txid);
    }

    let data = client.get_account_data(&market).await?;
    let header =
        bytemuck::try_from_bytes::<MarketHeader>(&data[..std::mem::size_of::<MarketHeader>()])
            .map_err(|_| anyhow::Error::msg("Failed to parse Phoenix market header"))?;

    println!("Quote Params: {:#?}", params);

    loop {
        let fair_price = {
            let response = reqwest::get(format!(
                "https://api.coinbase.com/v2/prices/{}/spot",
                ticker
            ))
            .await?
            .json::<serde_json::Value>()
            .await?;

            f64::from_str(response["data"]["amount"].as_str().unwrap())?
        };

        println!("Fair price: {}", fair_price);

        let args = UpdateQuotesInstruction {
            params: OrderParams {
                fair_price_in_quote_atoms_per_raw_base_unit: (fair_price * 1e6) as u64,
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
