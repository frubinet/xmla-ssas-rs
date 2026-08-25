// SPDX-License-Identifier: MPL-2.0

use clap::{ArgGroup, Args, Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use xmla_ssas_rs::connection::{NtlmCredentials, SsasTcpConnectionOptions};

#[derive(Debug, Parser)]
#[command(
    name = "xmla-ssas",
    version,
    about = "CLI client for SQL Server Analysis Services (SSAS)"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Check whether an SSAS XMLA endpoint is reachable
    Probe(ProbeArgs),

    /// Discover metadata exposed by an SSAS server
    Discover(DiscoverArgs),

    /// Execute an MDX query and write the result as CSV
    Query(QueryArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ProbeArgs {
    #[command(flatten)]
    pub(crate) endpoint: EndpointArgs,
}

#[derive(Debug, Args)]
pub(crate) struct DiscoverArgs {
    #[command(subcommand)]
    pub(crate) command: DiscoverCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum DiscoverCommand {
    /// List the catalogs available to the authenticated user
    Catalogs(DiscoverCatalogsArgs),
}

#[derive(Debug, Args)]
pub(crate) struct DiscoverCatalogsArgs {
    #[command(flatten)]
    pub(crate) endpoint: EndpointArgs,

    #[command(flatten)]
    pub(crate) credentials: CredentialsArgs,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("query_input")
        .required(true)
        .args(["mdx", "file"])
))]
pub(crate) struct QueryArgs {
    #[command(flatten)]
    pub(crate) endpoint: EndpointArgs,

    #[command(flatten)]
    pub(crate) credentials: CredentialsArgs,

    /// Catalog in which to execute the query
    #[arg(long, env = "SSAS_CATALOG")]
    pub(crate) catalog: String,

    /// MDX query text
    #[arg(long, value_name = "MDX")]
    pub(crate) mdx: Option<String>,

    /// Read the MDX query from a file
    #[arg(long, value_name = "PATH")]
    pub(crate) file: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct EndpointArgs {
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
pub(crate) struct CredentialsArgs {
    /// Windows domain used for NTLM authentication
    #[arg(long, env = "SSAS_DOMAIN")]
    domain: String,

    /// Username used for NTLM authentication
    #[arg(short, long, env = "SSAS_USERNAME")]
    username: String,
}

impl EndpointArgs {
    pub(crate) fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub(crate) fn connection_options(self) -> SsasTcpConnectionOptions {
        SsasTcpConnectionOptions::new(self.host, self.port)
            .with_connect_timeout(Duration::from_secs(self.connect_timeout))
            .with_read_timeout(Duration::from_secs(self.read_timeout))
    }
}

impl CredentialsArgs {
    pub(crate) fn with_password(self, password: String) -> NtlmCredentials {
        NtlmCredentials {
            domain: self.domain,
            username: self.username,
            password,
        }
    }
}
