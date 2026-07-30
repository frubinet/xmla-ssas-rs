// SPDX-License-Identifier: MPL-2.0

mod common;

use roxmltree::Document;
use std::env;
use xmla_ssas_rs::connection::{NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions};

#[test]
#[ignore = "requires a running SSAS server"]
fn execute_adv_works_total_sales_amount() -> Result<(), Box<dyn std::error::Error>> {
    // Authenticates and discovers catalogs
    common::init_logging();

    let tcp_options = SsasTcpConnectionOptions::new(
        env::var("SSAS_HOST")?,
        env::var("SSAS_PORT")?.parse::<u16>()?,
    );
    let credentials = NtlmCredentials {
        domain: env::var("SSAS_DOMAIN")?,
        username: env::var("SSAS_USERNAME")?,
        password: env::var("SSAS_PASSWORD")?,
    };
    let catalog = env::var("SSAS_CATALOG")?;
    let mut connection = SsasTcpConnection::connect(tcp_options, credentials)?;
    let xml = connection.execute(
        "SELECT [Measures].[Sales Amount] ON 0 FROM [Adventure Works]",
        catalog,
    )?;
    println!("{}", xml);
    let document = Document::parse(xml.as_str())?;
    let cell_data = document
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "CellData");
    assert!(cell_data.is_some());
    let cells = cell_data
        .unwrap()
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "FmtValue");
    for cell in cells {
        assert!(cell.text().is_some());
        println!("cell: {}", cell.text().unwrap());
    }
    Ok(())
}
