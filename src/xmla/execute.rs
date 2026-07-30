// SPDX-License-Identifier: MPL-2.0

use crate::connection::XmlaError;
use crate::xmla::props::XmlaProperties;
use crate::xmla::soap::ToXml;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use std::io;

pub struct XmlaExecute {
    pub query: String,
    pub properties: XmlaProperties,
}

impl XmlaExecute {
    pub fn new(query: impl Into<String>, catalog: impl Into<String>) -> Result<Self, XmlaError> {
        let mut properties = XmlaProperties::default();
        properties.add(XmlaProperties::CATALOG, catalog.into())?;
        Ok(Self {
            query: query.into(),
            properties,
        })
    }
}

impl ToXml for XmlaExecute {
    fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> io::Result<()> {
        //     <Execute xmlns="urn:schemas-microsoft-com:xml-analysis">
        //       <Command>
        //         <Statement>select Measures.members on 0 from [Adventure Works]</Statement>
        //       </Command>
        //       <Properties>
        //         <PropertyList>
        //           <Catalog>Adventure Works</Catalog>
        //         </PropertyList>
        //       </Properties>
        //     </Execute>
        let mut execute = BytesStart::new("Execute");
        execute.push_attribute(("xmlns", "urn:schemas-microsoft-com:xml-analysis"));
        writer.write_event(Event::Start(execute))?;
        writer.write_event(Event::Start(BytesStart::new("Command")))?;
        writer.write_event(Event::Start(BytesStart::new("Statement")))?;
        writer.write_event(Event::Text(BytesText::new(self.query.as_str())))?;
        writer.write_event(Event::End(BytesEnd::new("Statement")))?;
        writer.write_event(Event::End(BytesEnd::new("Command")))?;
        self.properties.to_xml(writer)?;
        writer.write_event(Event::End(BytesEnd::new("Execute")))?;
        Ok(())
    }
}
