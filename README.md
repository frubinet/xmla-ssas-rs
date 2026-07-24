# xmla-ssas-rs

Native Rust client for Microsoft SQL Server Analysis Services using XMLA over
TCP/DIME.

> Status: Early development: the API and protocol support are incomplete and may change.

## Current support

- NTLM authentication and message encryption
- XMLA `Discover` requests, returning the SOAP result, no Discover response parsing yet.

## Usage

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

    let mut connection = SsasTcpConnection::connect(options, credentials)?;
    let response = connection.discover("DBSCHEMA_CATALOGS".into())?;
    println!("{response}");

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
