use crate::config::{Config as PhoenixConfig, PhoenixOnChainMMConfig};
use crate::utils::get_pomm_config;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use phoenix_onchain_mm::accounts::Initialize as InitializeAccounts;
use phoenix_onchain_mm::instruction::Initialize as InitializeInstruction;
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
    // TODO: check have cliam sate
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;
        dbg!(&phoneix_config);

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;
        let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

        let PhoenixOnChainMMConfig {
            market,
            ticker: _,
            quote_edge_in_bps,
            quote_size,
            quote_refresh_frequency_in_ms: _,
            price_improvement_behavior,
            post_only,
        } = phoneix_config.phoenix;

        let (strategy_key, _bump_seed) = Pubkey::find_program_address(
            &[b"phoenix", payer.pubkey().as_ref(), market.as_ref()],
            &phoenix_onchain_mm::id(),
        );
        dbg!(&strategy_key);

        let price_improvement = match price_improvement_behavior.as_str() {
            "Join" | "join" => PriceImprovementBehavior::Join,
            "Dime" | "dime" => PriceImprovementBehavior::Dime,
            "Ignore" | "ignore" => PriceImprovementBehavior::Ignore,
            _ => PriceImprovementBehavior::Join,
        };
        dbg!(price_improvement);

        let params = StrategyParams {
            quote_edge_in_bps: Some(quote_edge_in_bps),
            quote_size_in_quote_atoms: Some(quote_size),
            price_improvement_behavior: Some(price_improvement),
            post_only: Some(post_only),
        };
        dbg!(&params);

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
        dbg!(&transaction);
        let txid = client.send_and_confirm_transaction(&transaction).await?;
        dbg!(&txid);

        println!(
            "Creating strategy account: https://beta.solscan.io/tx/{}?cluster=devnet",
            txid
        );
        Ok(())
    }
}
