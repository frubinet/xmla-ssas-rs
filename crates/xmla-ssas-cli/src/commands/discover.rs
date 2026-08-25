// SPDX-License-Identifier: MPL-2.0

use super::connect;
use crate::args::{DiscoverArgs, DiscoverCatalogsArgs, DiscoverCommand};
use std::error::Error;
use std::io;
use xmla_ssas_rs::xmla::XmlaRestrictions;

pub(crate) fn run(args: DiscoverArgs) -> Result<(), Box<dyn Error>> {
    match args.command {
        DiscoverCommand::Catalogs(args) => catalogs(args),
    }
}

fn catalogs(args: DiscoverCatalogsArgs) -> Result<(), Box<dyn Error>> {
    let mut connection = connect(args.endpoint, args.credentials)?;
    let response = connection.discover("DBSCHEMA_CATALOGS", &XmlaRestrictions::default())?;

    println!("Catalogs:");
    for row in response.rows() {
        let catalog = row.get("CATALOG_NAME").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "expected CATALOG_NAME attribute in row",
            )
        })?;
        println!("\t{catalog}");
    }

    Ok(())
}
