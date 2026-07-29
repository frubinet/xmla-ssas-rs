# xmla-ssas-rs

Native Rust client for Microsoft SQL Server Analysis Services using XMLA over
TCP/DIME. Based on MS-SSAS specification [v20260525](https://sqlprotocoldocs-cgcjdngdb5dee9c6.b02.azurefd.net/MS-SSAS/%5BMS-SSAS%5D-260525.pdf).

> Status: Early development: the API and protocol support are incomplete and may change.

## Current support

- NTLM authentication and message encryption
- XMLA `Discover` requests with restrictions and parsed rowset responses.

## Usage

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

## Development

After installing [pre-commit](https://pre-commit.com/), enable the formatting hook:

```sh
pre-commit install
```

## License

Licensed under the [Mozilla Public License 2.0](LICENSE).
