use crate::config::PhoenixOnChainMMConfig;
use crate::utils::get_pomm_config;
use ellipsis_client::EllipsisClient;
use phoenix::program::accounts::MarketHeader;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::rpc_filter::Memcmp;
use solana_client::rpc_filter::MemcmpEncodedBytes;
use solana_client::rpc_filter::RpcFilterType;
use solana_program::keccak;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use std::mem::size_of;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct GetMarketAddress {
    #[structopt(long, default_value = "5")]
    pub levels: usize,
    #[structopt(long, default_value = "4")]
    pub precision: usize,
}

fn get_discriminant(type_name: &str) -> u64 {
    u64::from_le_bytes(
        keccak::hashv(&[phoenix::ID.as_ref(), type_name.as_bytes()]).as_ref()[..8]
            .try_into()
            .unwrap(),
    )
}

// getting market data from the blockchain (devnet)
impl GetMarketAddress {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let client = EllipsisClient::from_rpc(
            RpcClient::new_with_commitment(rpc_enpoint, commitment),
            &payer,
        )?;

        let PhoenixOnChainMMConfig { ticker, .. } = phoneix_config.phoenix;

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
        let mut market_address: Vec<Pubkey> = vec![];

        for (market_pubkey, account) in program_accounts {
            let account_cloned = account.clone();
            // MarketHeader is fixed size; split the market account bytes into header bytes and market bytes
            let (header_bytes, _) = account_cloned.data.split_at(size_of::<MarketHeader>());

            // deserialize the header
            let header = bytemuck::try_from_bytes::<MarketHeader>(header_bytes).unwrap();

            if header.base_params.mint_key == generic_token_faucet::get_mint_address(&ticker.base)
                || header.base_params.mint_key == spl_token::native_mint::id()
            {
                market_address.push(market_pubkey);
            }
        }

        if market_address.is_empty() {
            println!("No {} market found", ticker);
            return Ok(());
        }

        println!("Getting {} order book", ticker);
        for market in market_address {
            println!("Market pubkey: {:?}", market);
        }

        Ok(())
    }
}
