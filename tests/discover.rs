// SPDX-License-Identifier: MPL-2.0

mod common;

use std::env;
use common::pretty_xml;
use xmla_ssas_rs::connection::{NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions};

#[test]
#[ignore = "requires a running SSAS server"]
fn connection_auth_and_discover_catalogs() -> Result<(), Box<dyn std::error::Error>> {
    // Authenticates and discovers catalogs
    let tcp_options = SsasTcpConnectionOptions::new(
        env::var("SSAS_HOST")?,
        env::var("SSAS_PORT")?.parse::<u16>()?,
    );
    let credentials = NtlmCredentials {
        domain: env::var("SSAS_DOMAIN")?,
        username: env::var("SSAS_USERNAME")?,
        password: env::var("SSAS_PASSWORD")?,
    };
    let mut connection = SsasTcpConnection::connect(tcp_options, credentials)?;
    let xml = connection.discover("DBSCHEMA_CATALOGS".to_string())?;
    println!("{}", pretty_xml(xml.as_str())?);
    Ok(())
}
