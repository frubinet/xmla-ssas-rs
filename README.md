# xmla-ssas-rs

Native Rust client for Microsoft SQL Server Analysis Services using XMLA over
TCP/DIME. Based on MS-SSAS specification [v20260525](https://sqlprotocoldocs-cgcjdngdb5dee9c6.b02.azurefd.net/MS-SSAS/%5BMS-SSAS%5D-260525.pdf).

> Status: Early development: the API and protocol support are incomplete and may change.

## Current support

- NTLM authentication and message encryption
- Compressed responses.
- XMLA `Discover` requests with restrictions and parsed rowset responses.
- XMLA `Execute` requests with result parsing to XmlaDataset objects.

## Not supported yet
- Compressed requests
- Binary XML

## Usage

### Discover Catalogs
```rust
use xmla_ssas_rs::connection::{
    NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions,
};
use xmla_ssas_rs::xmla::XmlaRestrictions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SsasTcpConnectionOptions::new("ssas.example.com", 2383);
    let credentials = NtlmCredentials {
        domain: "DOMAIN".into(),
        username: "user".into(),
        password: "password".into(),
    };

    let mut connection = SsasTcpConnection::connect(options, credentials)?;
    let response = connection.discover("DBSCHEMA_CATALOGS".to_string(), &XmlaRestrictions::default())?;
    let catalog_names: Vec<String> = response
        .rows()
        .filter_map(|row| row.get("CATALOG_NAME").cloned())
        .collect();
    println!("Catalogs: {:?}", catalog_names);

    Ok(())
}
```

### Execute Query
```rust
use xmla_ssas_rs::connection::{
    NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SsasTcpConnectionOptions::new("ssas.example.com", 2383);
    let credentials = NtlmCredentials {
        domain: "DOMAIN".into(),
        username: "user".into(),
        password: "password".into(),
    };
    let catalog = "AdvWorks";
    let mut connection = SsasTcpConnection::connect(options, credentials)?;
    let dataset = connection.execute(
        "SELECT {[Measures].[Sales Amount], [Measures].[Reseller Sales Amount]} ON COLUMNS FROM [Adventure Works]",
        catalog,
    )?;
    let column_count = dataset.column_count();
    let row_count = dataset.row_count();
    println!("(0,0): {}", dataset.cell_formatted_value_at(0, 0).unwrap());
    println!("(1,0): {}", dataset.cell_formatted_value_at(1, 0).unwrap());
    Ok(())
}
```

## Development

After installing [pre-commit](https://pre-commit.com/), enable the formatting hook:

```sh
pre-commit install
```

## License

Licensed under the [Mozilla Public License 2.0](LICENSE).
