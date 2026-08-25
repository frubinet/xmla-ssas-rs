// SPDX-License-Identifier: MPL-2.0

use clap::{Parser, Subcommand};
use std::error::Error;
use std::process::ExitCode;
use std::time::Duration;
use xmla_ssas_rs::connection::{SsasTcpConnection, SsasTcpConnectionOptions};

#[derive(Debug, Parser)]
#[command(
    name = "xmla-ssas",
    version,
    about = "CLI client for SQL Server Analysis Services (SSAS)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check whether an SSAS XMLA endpoint is reachable
    Probe {
        /// SSAS server hostname or IP address
        host: String,

        /// SSAS TCP port
        #[arg(short, long, default_value_t = 2383)]
        port: u16,

        /// Connection timeout in seconds
        #[arg(long, default_value_t = 10)]
        connect_timeout: u64,

        /// Response timeout in seconds
        #[arg(long, default_value_t = 60)]
        read_timeout: u64,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Probe {
            host,
            port,
            connect_timeout,
            read_timeout,
        } => {
            let options = SsasTcpConnectionOptions::new(&host, port)
                .with_connect_timeout(Duration::from_secs(connect_timeout))
                .with_read_timeout(Duration::from_secs(read_timeout));

            SsasTcpConnection::probe(options)?;

            println!("SSAS endpoint is reachable at {host}:{port}");
        }
    }

    Ok(())
}
