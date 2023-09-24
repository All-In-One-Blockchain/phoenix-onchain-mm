pub mod auto;
pub mod cancle;
pub mod initialize;
pub mod listen_balance;
pub mod update_quotes;

use structopt::StructOpt;

use auto::Auto;
use cancle::Cancle;
use initialize::Initialize;
use listen_balance::ListenBalance;
use update_quotes::UpdateQuotes;

#[derive(Debug, StructOpt)]
pub enum Command {
    /// auto generate config.toml file to ~/.config/pomm/config.toml
    #[structopt(name = "auto")]
    Auto(Auto),
    /// initialize Phoenix onchain Maket Maker and Claim Market Sate
    #[structopt(name = "init")]
    Initialize(Initialize),
    /// update quotes
    #[structopt(name = "update-quotes")]
    UpdateQuotes(UpdateQuotes),
    /// cancle all orders
    #[structopt(name = "cancle")]
    Cancle(Cancle),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "pomm")]
pub struct PhoneixOnChainMMCli {
    #[structopt(subcommand)]
    pub command: Command,
}

impl PhoneixOnChainMMCli {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Auto(auto) => {
                let config_path = auto.run();
                println!("ConfigPath: {:?}", config_path);
                Ok(())
            }
            Command::Initialize(initialize) => initialize.run().await,
            Command::UpdateQuotes(update_quotes) => update_quotes.run().await,
            Command::Cancle(cancle) => cancle.run().await,
        }
    }
}
