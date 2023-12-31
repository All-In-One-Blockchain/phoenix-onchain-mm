use crate::config::PhoenixOnChainMMConfig;
use crate::constant::{PHOENIX_ONCHAIN_MM_ORACLE_SEED, PHOENIX_ONCHAIN_MM_STRATEGY_SEED};
use crate::ids;
use crate::utils::get_pomm_config;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use phoenix_onchain_mm::accounts::Initialize as InitializeAccounts;
use phoenix_onchain_mm::instruction::Initialize as InitializeInstruction;
use phoenix_onchain_mm::oracle::OracleConfig;
use phoenix_onchain_mm::PriceImprovementBehavior;
use phoenix_onchain_mm::StrategyParams;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Initialize {}

impl Initialize {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;
        let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

        let PhoenixOnChainMMConfig {
            market,
            quote_edge_in_bps,
            quote_size,
            price_improvement_behavior,
            post_only,
            ..
        } = phoneix_config.phoenix.clone();

        let base_account = phoneix_config.phoenix.get_base_oracle_account()?;
        let quote_account = phoneix_config.phoenix.get_quote_oracle_account()?;

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
                oracle_base_account: base_account,
                oracle_quote_account: quote_account,
            },
        };

        let initialize_data = InitializeInstruction { params };
        let initialize_accounts = InitializeAccounts {
            phoenix_strategy: strategy_key,
            oracle_account,
            market,
            user: payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        };

        let ix = Instruction {
            program_id: ids::phoenix_onchain_mm_program::id(),
            accounts: initialize_accounts.to_account_metas(None),
            data: initialize_data.data(),
        };

        let blockhash = client.get_latest_blockhash().await?;

        let transaction =
            Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
        match client.send_and_confirm_transaction(&transaction).await {
            Ok(txid) => {
                println!(
                    "Creating strategy account: https://explorer.solana.com/tx/{}?cluster=devnet",
                    txid
                );
            }
            Err(e) => {
                println!("Initialize Error: {:#?}", e);
                return Ok(());
            }
        }

        Ok(())
    }
}
