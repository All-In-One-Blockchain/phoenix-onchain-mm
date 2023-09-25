pub mod airdrop_base_and_quote;
pub mod auto;
pub mod cancle;
pub mod fetch_market_event;
pub mod grpc;
pub mod initialize;
pub mod listen_balance;
pub mod update_quotes;
pub mod view_state_order_book;

use airdrop_base_and_quote::AirdropBaseAndQuote;
use auto::Auto;
use cancle::Cancle;
use fetch_market_event::FetchMarketEvent;
use initialize::Initialize;
use listen_balance::ListenBalance;
use structopt::StructOpt;
use update_quotes::UpdateQuotes;
use view_state_order_book::ViewStateOrderBook;

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
    /// listen balance
    #[structopt(name = "listen-balance")]
    ListenBalance(ListenBalance),
    /// airdrop base and quote token
    #[structopt(name = "airdrop")]
    AirDropBaseAndQuote(AirdropBaseAndQuote),
    /// fetch market event
    #[structopt(name = "fetch-market-event")]
    FetchMarketEvent(FetchMarketEvent),
    /// grpc
    #[structopt(name = "grpc")]
    Grpc(grpc::Grpc),
    /// view state order book
    #[structopt(name = "view-state-order-book")]
    ViewStateOrderBook(ViewStateOrderBook),
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
            Command::ListenBalance(listen_balance) => listen_balance.run().await,
            Command::AirDropBaseAndQuote(airdrop) => airdrop.run().await,
            Command::FetchMarketEvent(fetch_market_event) => fetch_market_event.run().await,
            Command::Grpc(grpc) => grpc.run().await,
            Command::ViewStateOrderBook(view_state_order_book) => view_state_order_book.run().await,
        }
    }
}
