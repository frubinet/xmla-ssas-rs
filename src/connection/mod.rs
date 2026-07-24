//! TCP connection management and authentication for Analysis Services.
mod connection;
mod error;

pub use connection::{NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions};
pub use error::{Result, XmlaError};
