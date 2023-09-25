use crate::utils::get_pomm_config;
use ellipsis_client::EllipsisClient;
use phoenix::program::accounts::MarketHeader;
use phoenix::program::dispatch_market::load_with_dispatch;
use phoenix_sdk::sdk_client::SDKClient;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::rpc_filter::Memcmp;
use solana_client::rpc_filter::MemcmpEncodedBytes;
use solana_client::rpc_filter::RpcFilterType;
use solana_program::keccak;
use solana_sdk::commitment_config::CommitmentConfig;
use std::mem::size_of;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListAllMarket {}

fn get_discriminant(type_name: &str) -> u64 {
    u64::from_le_bytes(
        keccak::hashv(&[phoenix::ID.as_ref(), type_name.as_bytes()]).as_ref()[..8]
            .try_into()
            .unwrap(),
    )
}

// getting market data from the blockchain (devnet)
impl ListAllMarket {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let _sdk = SDKClient::new(&payer, &rpc_enpoint).await?;

        let client = EllipsisClient::from_rpc(
            RpcClient::new_with_commitment(rpc_enpoint, commitment),
            &payer,
        )?;

        let market_discriminant = get_discriminant("phoenix::program::accounts::MarketHeader");

        // Fetch all markets
        // Memcmp encoding field is deprecated
        #[allow(deprecated)]
        let program_accounts = client
            .get_program_accounts_with_config(
                &phoenix::ID,
                RpcProgramAccountsConfig {
                    filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
                        offset: 0,
                        bytes: MemcmpEncodedBytes::Bytes(
                            market_discriminant.to_le_bytes().to_vec(),
                        ),
                        encoding: None,
                    })]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        commitment: Some(CommitmentConfig::confirmed()),
                        ..RpcAccountInfoConfig::default()
                    },

                    ..RpcProgramAccountsConfig::default()
                },
            )
            .await?;

        println!("Found {} markets", program_accounts.len());
        // let mut sol_usdc_market: Option<Pubkey> = None;

        for (market_pubkey, account) in program_accounts {
            let account_cloned = account.clone();
            // MarketHeader is fixed size; split the market account bytes into header bytes and market bytes
            let (header_bytes, market_bytes) =
                account_cloned.data.split_at(size_of::<MarketHeader>());

            // deserialize the header
            let header = bytemuck::try_from_bytes::<MarketHeader>(header_bytes).unwrap();

            // use params from the header to deserialize the market
            let _market = load_with_dispatch(&header.market_size_params, market_bytes)
                .unwrap()
                .inner;

            println!(
                "Pubkey: {:?}, Quote: {:?}, Base: {:?}",
                market_pubkey, header.quote_params.mint_key, header.base_params.mint_key
            );
        }
        Ok(())
    }
}
