
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![feature(trivial_bounds)]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(async_closure)]
#![feature(never_type)]









use nomic::network::Network;
//use nomic::network::InnerConfig;
use nomic::app::InnerApp;
use eyre::Result; 
use clap::Parser;
use clap::Args;


use nomic::network::Config;
use nomic::orga::coins::Address;




#[derive(Parser, Debug)]
pub struct BalanceCmd {
    /// The address to show the balance of. If not provided, the balance of the
    /// current wallet address is shown.
    address: Option<Address>,

    #[clap(flatten)]
    config: nomic::network::Config,
}


async fn get_balance(address: Option<Address>) -> Result<()> {

    let network = Network::Testnet;
    let inner_config = network.config();

    let mut app = InnerApp::default();


    // Create a Config instance using InnerConfig
    let config = Config {
        args: inner_config, // Pass InnerConfig to Config
    };

    let client = config.client();

    // Use `my_address` to get the current address if not provided
    let address = address.unwrap_or_else(my_address);
    println!("Querying balance for address: {}", address);

    // Query the balance
    let nom_balance = client.query(|app| app.accounts.balance(address)).await?;
    println!("Balance in NOM: {} NOM", nom_balance);

    let nbtc_balance = client.query(|app| app.bitcoin.accounts.balance(address)).await?;
    println!("Balance in NBTC: {} NBTC", nbtc_balance);

    let escrowed_balance = client.query(|app| app.escrowed_nbtc(address)).await?;
    println!("Escrowed balance in NBTC: {} IBC-escrowed NBTC", escrowed_balance);

    Ok(())
}
