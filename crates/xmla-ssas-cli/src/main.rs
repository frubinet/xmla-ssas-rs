// SPDX-License-Identifier: MPL-2.0

use clap::{Args, Parser, Subcommand};
use std::env;
use std::error::Error;
use std::io;
use std::process::ExitCode;
use std::time::Duration;
use xmla_ssas_rs::connection::{NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions};
use xmla_ssas_rs::xmla::XmlaRestrictions;

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
        #[command(flatten)]
        endpoint: EndpointArgs,
    },

    /// Discover metadata exposed by an SSAS server
    Discover {
        #[command(subcommand)]
        command: DiscoverCommand,
    },
}

#[derive(Debug, Subcommand)]
enum DiscoverCommand {
    /// List the catalogs available to the authenticated user
    Catalogs {
        #[command(flatten)]
        endpoint: EndpointArgs,

        #[command(flatten)]
        credentials: CredentialsArgs,
    },
}

#[derive(Debug, Args)]
struct EndpointArgs {
    /// SSAS server hostname or IP address
    #[arg(long, env = "SSAS_HOST")]
    host: String,

    /// SSAS TCP port
    #[arg(short, long, default_value_t = 2383, env = "SSAS_PORT")]
    port: u16,

    /// Connection timeout in seconds
    #[arg(long, default_value_t = 10)]
    connect_timeout: u64,

    /// Response timeout in seconds
    #[arg(long, default_value_t = 60)]
    read_timeout: u64,
}

#[derive(Debug, Args)]
struct CredentialsArgs {
    /// Windows domain used for NTLM authentication
    #[arg(long, env = "SSAS_DOMAIN")]
    domain: String,

    /// Username used for NTLM authentication
    #[arg(short, long, env = "SSAS_USERNAME")]
    username: String,
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
        Command::Probe { endpoint } => {
            let address = endpoint.address();
            let options = endpoint.connection_options();

            SsasTcpConnection::probe(options)?;

            println!("SSAS endpoint is reachable at {address}");
        }
        Command::Discover {
            command:
                DiscoverCommand::Catalogs {
                    endpoint,
                    credentials,
                },
        } => {
            let password = env::var("SSAS_PASSWORD").map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "SSAS_PASSWORD must be set for authenticated commands",
                )
            })?;
            let mut connection = SsasTcpConnection::connect(
                endpoint.connection_options(),
                credentials.with_password(password),
            )?;
            let response =
                connection.discover("DBSCHEMA_CATALOGS", &XmlaRestrictions::default())?;
            println!("Catalogs:");
            for row in response.rows() {
                let catalog = row.get("CATALOG_NAME").ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "catalog row does not contain CATALOG_NAME",
                    )
                })?;
                println!("\t{catalog}");
            }
        }
    }

    Ok(())
}

impl EndpointArgs {
    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn connection_options(self) -> SsasTcpConnectionOptions {
        SsasTcpConnectionOptions::new(self.host, self.port)
            .with_connect_timeout(Duration::from_secs(self.connect_timeout))
            .with_read_timeout(Duration::from_secs(self.read_timeout))
    }
}

impl CredentialsArgs {
    fn with_password(self, password: String) -> NtlmCredentials {
        NtlmCredentials {
            domain: self.domain,
            username: self.username,
            password,
        }
    }
}
