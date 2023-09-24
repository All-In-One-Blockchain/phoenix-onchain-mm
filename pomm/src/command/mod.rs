pub mod auto;
pub mod initialize;

use structopt::StructOpt;

use auto::Auto;
use initialize::Initialize;

#[derive(Debug, StructOpt)]
pub enum Command {
    /// auto generate config.toml file to ~/.config/pomm/config.toml
    #[structopt(name = "auto")]
    Auto(Auto),
    /// initialize Phoenix onchain Maket Maker and Claim Market Sate
    Initialize(Initialize),
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
        }
    }
}
