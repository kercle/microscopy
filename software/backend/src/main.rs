use clap::Parser;

use crate::cli::CliCommand;

mod camera;
mod cli;
mod control_app;
mod handlers;
mod parameters;
mod compute_node;

#[tokio::main]
async fn main() {
    let cli_command = CliCommand::parse();

    match cli_command {
        CliCommand::Serve(cmd) => {
            cmd.exec().await;
        }
    }
}
