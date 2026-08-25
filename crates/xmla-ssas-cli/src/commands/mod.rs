// SPDX-License-Identifier: MPL-2.0

pub(crate) mod discover;
pub(crate) mod probe;
pub(crate) mod query;

use crate::args::{CredentialsArgs, EndpointArgs};
use std::env;
use std::error::Error;
use std::io;
use xmla_ssas_rs::connection::SsasTcpConnection;

fn connect(
    endpoint: EndpointArgs,
    credentials: CredentialsArgs,
) -> Result<SsasTcpConnection, Box<dyn Error>> {
    let password = env::var("SSAS_PASSWORD").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SSAS_PASSWORD must be set for authenticated commands",
        )
    })?;

    Ok(SsasTcpConnection::connect(
        endpoint.connection_options(),
        credentials.with_password(password),
    )?)
}
