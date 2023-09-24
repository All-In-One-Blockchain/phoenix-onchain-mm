use crate::config::{Config as PhoenixConfig, PhoenixOnChainMMConfig};
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
    pub async fn run(&self) -> anyhow::Result<()> {
        let home_path = dirs::home_dir().ok_or(anyhow::anyhow!("can't open home dir"))?;
        let pomm_config_path = home_path.join(".config").join("pomm");
        let config_path = pomm_config_path.join("config.toml");

        // 读取配置文件
        let config_str = std::fs::read_to_string(config_path).unwrap();
        // 解析配置文件
        let phoneix_config: PhoenixConfig = toml::from_str(&config_str).unwrap();

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;
        let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

        let sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

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
        println!(
            "Creating strategy account: https://beta.solscan.io/tx/{}?cluster=devnet",
            txid
        );

        // - Create the associated token account, if needed, for both base and quote tokens
        // - Claim a seat on the market, if needed
        let set_claim_marke_ix = sdk.get_maker_setup_instructions_for_market(&market).await?;

        let sig = sdk
            .client
            .sign_send_instructions(set_claim_marke_ix, vec![])
            .await?;

        println!(
            "Link to view transaction: https://beta.solscan.io/tx/{}?cluster=devnet",
            sig
        );
        Ok(())
    }
}
