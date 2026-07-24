use std::env;
use xmla_ssas_rs::connection::{SsasTcpConnection, SsasTcpConnectionOptions};

#[test]
#[ignore = "requires a running SSAS server"]
fn test_connectivity() -> Result<(), Box<dyn std::error::Error>> {
    // Tests connectivity to a given host/port
    let tcp_options = SsasTcpConnectionOptions::new(
        env::var("SSAS_HOST")?,
        env::var("SSAS_PORT")?.parse::<u16>()?,
    );
    SsasTcpConnection::test_connectivity(tcp_options)?;
    Ok(())
}