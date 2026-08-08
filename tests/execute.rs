// SPDX-License-Identifier: MPL-2.0

mod common;

use approx::assert_abs_diff_eq;
use std::env;
use xmla_ssas_rs::connection::{NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions};

#[test]
#[ignore = "requires a running SSAS server"]
fn execute_adv_works_total_sales_amount() -> Result<(), Box<dyn std::error::Error>> {
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
    let dataset = connection.execute(
        "SELECT [Measures].[Sales Amount] ON COLUMNS FROM [Adventure Works]",
        catalog,
    )?;
    let axis0 = dataset.axes().iter().find(|axis| axis.name() == "Axis0");
    assert!(axis0.is_some());
    let axis0 = axis0.unwrap();
    println!("axis0: {:?}", axis0);
    let cells = dataset.cells();
    println!("cells: {:?}", cells);
    let first_cell = dataset.cell(0);
    assert!(first_cell.is_some());
    let first_cell = first_cell.unwrap();
    println!("first_cell: {:?}", first_cell);
    Ok(())
}

#[test]
#[ignore = "requires a running SSAS server"]
fn execute_adv_works_total_multi_sales_amount() -> Result<(), Box<dyn std::error::Error>> {
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
    let dataset = connection.execute(
        "SELECT {[Measures].[Sales Amount], [Measures].[Reseller Sales Amount]} ON COLUMNS FROM [Adventure Works]",
        catalog,
    )?;
    let column_count = dataset.column_count();
    assert_eq!(column_count, 2);
    let row_count = dataset.row_count();
    assert_eq!(row_count, 1);
    let cell_0_0 = dataset.cell_at(0, 0);
    let cell_1_0 = dataset.cell_at(1, 0);
    assert!(cell_0_0.is_some());
    assert!(cell_1_0.is_some());
    let value_0_0 = dataset.cell_value_at(0, 0);
    let value_1_0 = dataset.cell_value_at(1, 0);
    assert!(value_0_0.is_some());
    assert!(value_1_0.is_some());
    assert_abs_diff_eq!(value_0_0.unwrap(), 109809274.20, epsilon = 0.01);
    assert_abs_diff_eq!(value_1_0.unwrap(), 80450596.98, epsilon = 0.01);
    let fmt_value_0_0 = dataset.cell_formatted_value_at(0, 0);
    let fmt_value_1_0 = dataset.cell_formatted_value_at(1, 0);
    assert!(fmt_value_0_0.is_some());
    assert!(fmt_value_1_0.is_some());
    assert_eq!(fmt_value_0_0.unwrap(), "$109,809,274.20");
    assert_eq!(fmt_value_1_0.unwrap(), "$80,450,596.98");
    let cell_1_1 = dataset.cell_at(1, 1);
    assert!(cell_1_1.is_none());
    let value_1_1 = dataset.cell_value_at(1, 1);
    assert!(value_1_1.is_none());
    Ok(())
}

#[test]
#[ignore = "requires a running SSAS server"]
fn execute_adv_works_countries_total_multi_sales_amount() -> Result<(), Box<dyn std::error::Error>>
{
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
    let dataset = connection.execute(
        "SELECT \
        {[Measures].[Reseller Order Quantity], [Measures].[Reseller Sales Amount]} \
        ON COLUMNS, \
        [Geography].[Country].CHILDREN ON ROWS
        FROM [Adventure Works]",
        catalog,
    )?;
    let column_count = dataset.column_count();
    assert_eq!(column_count, 2);
    let row_count = dataset.row_count();
    assert_eq!(row_count, 6);
    let captions = dataset
        .axis(1)
        .unwrap()
        .tuples()
        .iter()
        .map(|tuple| tuple.members().first().unwrap().caption())
        .collect::<Vec<_>>();
    println!("{:?}", captions);
    assert_eq!(
        captions,
        [
            "Australia",
            "Canada",
            "France",
            "Germany",
            "United Kingdom",
            "United States"
        ]
    );
    let value_0_5 = dataset.cell_value_at(0, 5);
    let fmt_value_0_5 = dataset.cell_formatted_value_at(0, 5);
    assert!(value_0_5.is_some());
    assert!(fmt_value_0_5.is_some());
    assert_abs_diff_eq!(value_0_5.unwrap(), 132748.0, epsilon = 0.01);
    assert_eq!(fmt_value_0_5.unwrap(), "132,748");
    let value_1_3 = dataset.cell_value_at(1, 3);
    let fmt_value_1_3 = dataset.cell_formatted_value_at(1, 3);
    assert!(value_1_3.is_some());
    assert!(fmt_value_1_3.is_some());
    assert_abs_diff_eq!(value_1_3.unwrap(), 1983988.04, epsilon = 0.01);
    assert_eq!(fmt_value_1_3.unwrap(), "$1,983,988.04");
    assert_eq!(dataset.cell_formatted_value_at(0, 0).unwrap(), "4,948");
    assert_eq!(dataset.cell_formatted_value_at(0, 1).unwrap(), "41,761");
    assert_eq!(dataset.cell_formatted_value_at(0, 2).unwrap(), "14,348");
    assert_eq!(dataset.cell_formatted_value_at(0, 3).unwrap(), "7,380");
    assert_eq!(dataset.cell_formatted_value_at(0, 4).unwrap(), "13,193");
    assert_eq!(dataset.cell_formatted_value_at(0, 5).unwrap(), "132,748");
    assert_eq!(
        dataset.cell_formatted_value_at(1, 0).unwrap(),
        "$1,594,335.38"
    );
    assert_eq!(
        dataset.cell_formatted_value_at(1, 1).unwrap(),
        "$14,377,925.60"
    );
    assert_eq!(
        dataset.cell_formatted_value_at(1, 2).unwrap(),
        "$4,607,537.94"
    );
    assert_eq!(
        dataset.cell_formatted_value_at(1, 3).unwrap(),
        "$1,983,988.04"
    );
    assert_eq!(
        dataset.cell_formatted_value_at(1, 4).unwrap(),
        "$4,279,008.83"
    );
    assert_eq!(
        dataset.cell_formatted_value_at(1, 5).unwrap(),
        "$53,607,801.21"
    );
    Ok(())
}
