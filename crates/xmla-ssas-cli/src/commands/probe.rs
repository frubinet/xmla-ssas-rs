// SPDX-License-Identifier: MPL-2.0

use crate::args::ProbeArgs;
use std::error::Error;
use xmla_ssas_rs::connection::SsasTcpConnection;

pub(crate) fn run(args: ProbeArgs) -> Result<(), Box<dyn Error>> {
    let address = args.endpoint.address();
    let options = args.endpoint.connection_options();

    SsasTcpConnection::probe(options)?;
    println!("SSAS endpoint is reachable at {address}");

    Ok(())
}
