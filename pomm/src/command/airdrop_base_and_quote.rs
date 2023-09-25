use crate::config::PhoenixOnChainMMConfig;
use crate::utils::create_airdrop_spl_ixs;
use crate::utils::get_pomm_config;
use solana_sdk::signer::Signer;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AirdropBaseAndQuote {}

impl AirdropBaseAndQuote {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (_, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

        let PhoenixOnChainMMConfig { market, .. } = phoneix_config.phoenix;

        // To test on devnet, (i) airdrop devnet SOL to the trader account, and (ii) airdrop tokens for the market's base and quote tokens.
        // These instructions only work on devnet.
        // (i) airdrop devnet SOL to the trader account. This step may not be needed if your trader keypair (from the above file_path) already has devnet SOL.
        // Below is an example of how to airdrop devnet SOL to the trader account. Commented out here because this method fails frequently on devnet.
        // Ensure that your trader keypair has devnet SOL to execute transactions.
        // sdk.client
        //     .request_airdrop(&trader.pubkey(), 1_000_000_000)
        //     .await
        //     .unwrap();

        // (ii) Airdrop tokens for the base and quote tokens for the supplied market, used for testing trades.
        // Uses the generic-token-faucet (https://github.com/Ellipsis-Labs/generic-token-faucet).
        let instructions = create_airdrop_spl_ixs(&sdk, &market, &payer.pubkey())
            .await
            .ok_or(anyhow::anyhow!("empty instruction!"))?;

        let setup_tx = sdk
            .client
            .sign_send_instructions(instructions, vec![])
            .await?;

        println!(
            "Setup tx: https://explorer.solana.com/tx/{}?cluster=devnet",
            setup_tx
        );

        Ok(())
    }
}
