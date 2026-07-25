// SPDX-License-Identifier: MPL-2.0

//! TCP connection management and authentication for Analysis Services.
mod error;
mod tcp;

pub use error::{Result, XmlaError};
pub use tcp::{NtlmCredentials, SsasTcpConnection, SsasTcpConnectionOptions};
