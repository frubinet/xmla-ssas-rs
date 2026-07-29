// SPDX-License-Identifier: MPL-2.0

mod common;

use std::env;
use xmla_ssas_rs::connection::{NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions};
use xmla_ssas_rs::xmla::XmlaRestrictions;

#[test]
#[ignore = "requires a running SSAS server"]
fn connection_auth_and_discover_catalogs() -> Result<(), Box<dyn std::error::Error>> {
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
    let mut connection = SsasTcpConnection::connect(tcp_options, credentials)?;
    let response = connection.discover(
        "DBSCHEMA_CATALOGS".to_string(),
        &XmlaRestrictions::default(),
    )?;
    let catalog_names: Vec<String> = response
        .rows()
        .filter_map(|row| row.get("CATALOG_NAME").cloned())
        .collect();
    assert!(!catalog_names.is_empty());
    println!("Catalogs: {:?}", catalog_names);
    Ok(())
}

#[test]
#[ignore = "requires a running SSAS server"]
fn connection_auth_and_discover_cubes_dimensions_measures() -> Result<(), Box<dyn std::error::Error>>
{
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
    let mut connection = SsasTcpConnection::connect(tcp_options, credentials)?;
    let response = connection.discover("DBSCHEMA_CATALOGS", &XmlaRestrictions::default())?;
    let catalog_names: Vec<String> = response
        .rows()
        .filter_map(|row| row.get("CATALOG_NAME").cloned())
        .collect();
    assert!(!catalog_names.is_empty());
    let catalog_name = &catalog_names[0];
    println!("Selected catalog: {:?}", catalog_name);

    let mut catalog_restrictions = XmlaRestrictions::default();
    catalog_restrictions.add("CATALOG_NAME", catalog_name)?;
    let response = connection.discover("MDSCHEMA_CUBES".to_string(), &catalog_restrictions)?;
    let cube_names: Vec<String> = response
        .rows()
        .filter_map(|row| row.get("CUBE_NAME").cloned())
        .collect();
    println!("cube names: {:?}", cube_names);
    assert!(!cube_names.is_empty());
    let cube_name = &cube_names[0];

    let mut cube_restrictions = XmlaRestrictions::default();
    cube_restrictions.add("CATALOG_NAME", catalog_name)?;
    cube_restrictions.add("CUBE_NAME", cube_name)?;
    let response = connection.discover("MDSCHEMA_DIMENSIONS", &cube_restrictions)?;
    let dimension_names: Vec<String> = response
        .rows()
        .filter_map(|row| row.get("DIMENSION_UNIQUE_NAME").cloned())
        .collect();
    println!("dimension names: {:?}", dimension_names);

    let response = connection.discover("MDSCHEMA_MEASURES", &cube_restrictions)?;
    let measure_names: Vec<String> = response
        .rows()
        .filter_map(|row| row.get("MEASURE_UNIQUE_NAME").cloned())
        .collect();
    println!("measure names: {:?}", measure_names);

    Ok(())
}
