use clap::Parser;
use core::FurDB;

use cli::Cli;
use cli::Commands;

use server::Server;

use error::ApplicationError;

mod cli;
mod core;
mod error;
mod server;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    dotenv::dotenv().ok();

    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let furdb_config = args.furdb_config;
    let furdb = FurDB::new(&furdb_config)?;

    match args.command {
        Commands::Serve(server_config) => {
            let server = Server::new(server_config, furdb);
            server.start().await?;
        }
    }

    Ok(())
}
