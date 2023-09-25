use crate::utils::get_pomm_config;
use ellipsis_client::grpc_client::transaction_subscribe;
use phoenix_sdk::sdk_client::SDKClient;
use solana_sdk::pubkey::Pubkey;
use structopt::StructOpt;
use tokio::{sync::mpsc::channel, try_join};

#[derive(Debug, StructOpt)]
pub struct Grpc {
    #[structopt(long)]
    x_token: String,

    /// Filter included accounts in transactions
    #[structopt(long)]
    accounts_to_include: Vec<Pubkey>,

    /// Filter excluded accounts in transactions
    #[structopt(long)]
    accounts_to_exclude: Vec<Pubkey>,
}

impl Grpc {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (_, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let sdk = SDKClient::new(&payer, &rpc_enpoint).await?;

        let (sender, mut receiver) = channel(10000);

        let x_token = self.x_token.clone();
        let accounts_to_include = self.accounts_to_include.clone();
        let accounts_to_exclude = self.accounts_to_exclude.clone();

        let market_data_sender = tokio::spawn(async move {
            transaction_subscribe(
                rpc_enpoint.clone(),
                Some(x_token),
                sender,
                accounts_to_include,
                accounts_to_exclude,
            )
            .await
        });

        let handler = tokio::spawn(async move {
            while let Some(transaction) = receiver.recv().await {
                let events = sdk.core.parse_events_from_transaction(&transaction);
                if let Some(events) = events {
                    if let Some(parsed_events) = sdk.parse_raw_phoenix_events(events).await {
                        for event in parsed_events {
                            println!("{:#?}", event);
                        }
                    }
                }
            }
        });

        match try_join!(market_data_sender, handler) {
            Ok(_) => {}
            Err(_) => {
                println!("Error");
            }
        }

        Ok(())
    }
}
