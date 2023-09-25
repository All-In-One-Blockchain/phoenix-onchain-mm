use crate::config::Config as PhoenixConfig;
use phoenix_sdk::sdk_client::SDKClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::state::Mint;

pub fn get_pomm_config() -> anyhow::Result<PhoenixConfig> {
    let home_path = dirs::home_dir().ok_or(anyhow::anyhow!("can't open home dir"))?;
    let pomm_config_path = home_path.join(".config").join("pomm");
    let config_path = pomm_config_path.join("config.toml");

    // 读取配置文件
    let config_str = std::fs::read_to_string(config_path)?;
    // 解析配置文件
    let phoneix_config: PhoenixConfig = toml::from_str(&config_str)?;

    Ok(phoneix_config)
}

// Only needed for devnet testing
pub async fn create_airdrop_spl_ixs(
    sdk_client: &SDKClient,
    market_pubkey: &Pubkey,
    recipient_pubkey: &Pubkey,
) -> Option<Vec<Instruction>> {
    // Get base and quote mints from market metadata
    let market_metadata = sdk_client.get_market_metadata(market_pubkey).await.ok()?;
    let base_mint = market_metadata.base_mint;
    let quote_mint = market_metadata.quote_mint;

    let mint_accounts = sdk_client
        .client
        .get_multiple_accounts(&[base_mint, quote_mint])
        .await
        .unwrap()
        .into_iter()
        .flatten()
        .collect::<Vec<Account>>();

    let base_mint_account = Mint::unpack(&mint_accounts[0].data).unwrap();

    let quote_mint_account = Mint::unpack(&mint_accounts[1].data).unwrap();

    let base_mint_authority = base_mint_account.mint_authority.unwrap();
    let quote_mint_authority = quote_mint_account.mint_authority.unwrap();

    let mint_authority_accounts = sdk_client
        .client
        .get_multiple_accounts(&[base_mint_authority, quote_mint_authority])
        .await
        .unwrap()
        .into_iter()
        .flatten()
        .collect::<Vec<Account>>();

    // If either the base or quote mint authority accounts (PDAs) are not owned by the devnet token faucet program, abort minting
    if mint_authority_accounts[0].owner != generic_token_faucet::id()
        || mint_authority_accounts[1].owner != generic_token_faucet::id()
    {
        return None;
    }

    // Get or create the ATA for the recipient. If doesn't exist, create token account
    let mut instructions = vec![];

    let recipient_ata_base =
        spl_associated_token_account::get_associated_token_address(recipient_pubkey, &base_mint);

    let recipient_ata_quote =
        spl_associated_token_account::get_associated_token_address(recipient_pubkey, &quote_mint);

    let recipient_ata_accounts = sdk_client
        .client
        .get_multiple_accounts(&[recipient_ata_base, recipient_ata_quote])
        .await
        .unwrap();

    if recipient_ata_accounts[0].is_none() {
        println!("Error retrieving base ATA. Creating base ATA");
        instructions.push(create_associated_token_account(
            &sdk_client.client.payer.pubkey(),
            recipient_pubkey,
            &base_mint,
            &spl_token::id(),
        ))
    };

    if recipient_ata_accounts[1].is_none() {
        println!("Error retrieving quote ATA. Creating quote ATA");
        instructions.push(create_associated_token_account(
            &sdk_client.client.payer.pubkey(),
            recipient_pubkey,
            &quote_mint,
            &spl_token::id(),
        ))
    };

    // Finally, mint the base and quote tokens to the recipient. The recipient's ATAs will be automatically derived.
    instructions.push(generic_token_faucet::airdrop_spl_with_mint_pdas_ix(
        &generic_token_faucet::id(),
        &base_mint,
        &base_mint_authority,
        recipient_pubkey,
        (5000.0 * 1e9) as u64,
    ));

    instructions.push(generic_token_faucet::airdrop_spl_with_mint_pdas_ix(
        &generic_token_faucet::id(),
        &quote_mint,
        &quote_mint_authority,
        recipient_pubkey,
        (500000.0 * 1e6) as u64,
    ));

    Some(instructions)
}
