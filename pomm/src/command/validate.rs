use crate::config::PhoenixOnChainMMConfig;
use crate::config::Ticker;
use crate::utils::get_pomm_config;
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

#[derive(Debug, StructOpt)]
pub struct Validate {}

impl Validate {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

        let _sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

        let PhoenixOnChainMMConfig { market, ticker, .. } = phoneix_config.phoenix.clone();

        let base_account = phoneix_config.phoenix.get_base_oracle_account()?;
        let quote_account = phoneix_config.phoenix.get_quote_oracle_account()?;

        check_market_address_by_ticker(market, &client, ticker).await?;
        check_oracle_price_account(&client, base_account, quote_account).await?;

        Ok(())
    }
}

fn get_discriminant(type_name: &str) -> u64 {
    u64::from_le_bytes(
        keccak::hashv(&[phoenix::ID.as_ref(), type_name.as_bytes()]).as_ref()[..8]
            .try_into()
            .unwrap(),
    )
}

async fn check_market_address_by_ticker(
    maket_address: Pubkey,
    client: &RpcClient,
    ticker: Ticker,
) -> anyhow::Result<()> {
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
                    bytes: MemcmpEncodedBytes::Bytes(market_discriminant.to_le_bytes().to_vec()),
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
        Err(anyhow::anyhow!("No {} market found", ticker))
    } else {
        println!("Found {} {} markets", market_address.len(), ticker);
        if market_address.iter().any(|x| *x == maket_address) {
            println!("Market address {} is valid", maket_address);
            Ok(())
        } else {
            println!("Market address {} is invalid", maket_address);
            Err(anyhow::anyhow!(
                "Market address {} is invalid",
                maket_address
            ))
        }
    }
}

async fn check_oracle_price_account(
    client: &RpcClient,
    base_account: Pubkey,
    quote_account: Pubkey,
) -> anyhow::Result<()> {
    let _ = client.get_account(&base_account).await?;
    let _ = client.get_account(&quote_account).await?;
    Ok(())
}
