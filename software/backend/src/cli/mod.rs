mod serve;

use clap::Parser;

#[derive(Parser)]
pub enum CliCommand {
    Serve(serve::Command),
}
