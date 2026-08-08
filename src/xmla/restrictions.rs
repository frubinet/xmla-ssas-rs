// SPDX-License-Identifier: MPL-2.0

use crate::connection::XmlaError;
use crate::xmla::soap::ToXml;
use crate::xmla::values_map::XmlaValueMap;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use std::io;

#[derive(Debug, Default, Clone)]
pub struct XmlaRestrictions {
    values: XmlaValueMap,
}

impl XmlaRestrictions {
    pub const CATALOG_NAME: &'static str = "CATALOG_NAME";

    pub fn add(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), XmlaError> {
        self.values.add(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values.entries()
    }
}

impl ToXml for XmlaRestrictions {
    fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> io::Result<()> {
        writer.write_event(Event::Start(BytesStart::new("Restrictions")))?;
        writer.write_event(Event::Start(BytesStart::new("RestrictionList")))?;
        self.values.to_xml(writer)?;
        writer.write_event(Event::End(BytesEnd::new("RestrictionList")))?;
        writer.write_event(Event::End(BytesEnd::new("Restrictions")))?;
        Ok(())
    }
}
